#[cfg(not(target_os = "windows"))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}

#[cfg(target_os = "windows")]
fn main() -> windows_service::Result<()> {
    service::run()
}

#[cfg(target_os = "windows")]
mod service {
    use std::{env, ffi::OsString, sync::mpsc, time::Duration};
    use std::{path::PathBuf, fs::File, fs::OpenOptions, io::Write};
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };
    use std::{process, thread};
    use std::panic::catch_unwind;
    use std::process::Command;
    use egui::accesskit::DefaultActionVerb::Open;
    use sysinfo::Pid;
    use chrono::Local;

    const SERVICE_NAME: &str = "BackMeUp";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    // Generate the windows service boilerplate.
    // The boilerplate contains the low-level service entry function (ffi_service_main) that parses
    // incoming service arguments into Vec<OsString> and passes them to user defined service
    // entry (my_service_main).
    define_windows_service!(ffi_service_main, my_service_main);

    // Service entry function which is called on background thread by the system with service
    // parameters. There is no stdout or stderr at this point so make sure to configure the log
    // output to file if needed.
    pub fn my_service_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // Handle the error.
            eprintln!("Error while running the service: {:?}", _e);
            return;
        }
    }

    pub fn run_service() -> Result<()> {
        // Create a channel to be able to poll a stop event from the service worker loop.
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Define system service event handler that will be receiving service events.
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NoError even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Handle stop
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }

                // treat the UserEvent as a stop request
                ServiceControl::UserEvent(code) => {
                    if code.to_raw() == 130 {
                        shutdown_tx.send(()).unwrap();
                    }
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that service is running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // For demo purposes this service sends a UDP packet once a second.
        // let loopback_ip = IpAddr::from(LOOPBACK_ADDR);
        // let sender_addr = SocketAddr::new(loopback_ip, 0);
        // let receiver_addr = SocketAddr::new(loopback_ip, RECEIVER_PORT);
        // let msg = PING_MESSAGE.as_bytes();
        // let socket = UdpSocket::bind(sender_addr).unwrap();

        log_message("Servizio avviato 1 \n");

        loop {
            // Poll shutdown event.
            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                // Break the loop either upon stop or channel disconnect
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                // Continue work if no events were received within the timeout
                Err(mpsc::RecvTimeoutError::Timeout) => {log_message("Servizio avviato 2 \n"); ()},
            };

            log_message("Servizio avviato 3 \n");
            keep_backup_program_alive();
        }

        // Tell the system that service has stopped.
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    fn keep_backup_program_alive() {
        // Controlla se il processo è già in esecuzione
        if is_backup_program_running() {
            return;
        }

        // Avvia il programma nella sessione utente attiva
        if let Err(e) = launch_backup_program() {
            eprintln!("Errore nell'avviare backup_program: {:?}", e);
        }
    }

    fn is_backup_program_running() -> bool {
        // Usa un comando per verificare se il processo è attivo
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq backup_program.exe"])
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains("backup_program.exe"),
            Err(_) => false,
        }
    }

    fn launch_backup_program() -> std::io::Result<()> {
        // let mut output = Command::new("schtasks")
        //     .args([
        //         "/Create",
        //         "/SC",
        //         "ONCE",
        //         "/TN",
        //         "LaunchBackupProgram",
        //         "/TR",
        //         "C:/Users/maldimeriggio/Desktop/BackMeUp/bin/backup_program.exe",
        //         "/ST",
        //         "00:00", // Avvia subito
        //         "/RU",
        //         "SYSTEM",
        //         "/F",
        //     ])
        //     .output();
        //
        // match output {
        //     Ok(output) => {
        //         log_message(&format!(
        //             // "Creazione attività schtasks eseguita con successo. Output: {} \n",
        //             // String::from_utf8_lossy(&output.stdout)
        //             "***CREAZIONE*** \nStdout: {}\nStderr: {} \n",
        //             String::from_utf8_lossy(&output.stdout),
        //             String::from_utf8_lossy(&output.stderr)
        //         ));
        //     }
        //     Err(e) => {
        //         log_message(&format!("Errore durante l'esecuzione di schtasks: {} \n", e));
        //     }
        // }
        //
        // output = Command::new("schtasks")
        //     .args(["/Run", "/TN", "LaunchBackupProgram"])
        //     .output();
        //
        // match output {
        //     Ok(output) => {
        //         log_message(&format!(
        //             // "Lancio schtasks eseguito con successo. Output: {} \n",
        //             // String::from_utf8_lossy(&output.stdout)
        //             "***lancio*** \nStdout: {}\nStderr: {}",
        //             String::from_utf8_lossy(&output.stdout),
        //             String::from_utf8_lossy(&output.stderr)
        //         ));
        //     }
        //     Err(e) => {
        //         log_message(&format!("Errore durante l'esecuzione di schtasks: {} \n", e));
        //     }
        // }

        log_message("Iniziando launch_backup_program");

        let exe_dir = match env::current_exe() {
            Ok(path) => {
                log_message(&format!("Percorso exe corrente: {:?}", path));
                path.parent()
                    .ok_or_else(|| {
                        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "Parent directory not found");
                        log_message(&format!("Errore nel trovare la directory parent: {:?}", err));
                        err
                    })?
                    .to_path_buf()
            }
            Err(e) => {
                log_message(&format!("Errore nel trovare il percorso dell'executable: {:?}", e));
                return Err(e);
            }
        };

        let backup_program_path = exe_dir.join("backup_program.exe");
        log_message(&format!("Percorso backup program: {:?}", backup_program_path));

        if !backup_program_path.exists() {
            let err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("backup_program.exe non trovato in {:?}", backup_program_path)
            );
            log_message(&format!("Errore: {}", err));
            return Err(err);
        }

        // Ottieni username corrente
        let output = Command::new("query")
            .args(["session"])
            .output()?;

        let sessions = String::from_utf8_lossy(&output.stdout);
        let active_session = sessions.lines()
            .find(|line| line.contains("Active"))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No active session found"))?;

        let username = active_session
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not parse username"))?;

        log_message(&format!("Username trovato: {}", username));

        // Calcola l'orario di esecuzione un minuto nel futuro
        let start_time = Local::now() + chrono::Duration::minutes(1);
        let start_time_str = start_time.format("%H:%M").to_string();

        log_message(&format!("Creazione del task per l'orario: {}", start_time_str));

        // Crea il task specificando l'utente e il flag IT per l'interattività
        let create_result = Command::new("schtasks")
            .args([
                "/Create",
                "/TN", "BackupProgramLauncher",
                "/TR", &backup_program_path.to_string_lossy(),
                "/SC", "ONCE",
                "/ST", &start_time_str,
                "/RU", username,    // Specifica l'utente
                "/IT",             // Permetti l'interazione con il desktop
                "/F",             // Forza la sovrascrittura
                "/RL", "HIGHEST"  // Esegui con i privilegi più alti
            ])
            .output()?;

        log_message(&format!(
            "Risultato creazione task:\nStatus: {}\nStdout: {}\nStderr: {}",
            create_result.status,
            String::from_utf8_lossy(&create_result.stdout),
            String::from_utf8_lossy(&create_result.stderr)
        ));

        if !create_result.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create scheduled task: {}",
                        String::from_utf8_lossy(&create_result.stderr))
            ));
        }

        // Avvia il task immediatamente
        log_message("Avvio del task...");
        let run_result = Command::new("schtasks")
            .args([
                "/Run",
                "/TN", "BackupProgramLauncher"
            ])
            .output()?;

        log_message(&format!(
            "Risultato avvio task:\nStatus: {}\nStdout: {}\nStderr: {}",
            run_result.status,
            String::from_utf8_lossy(&run_result.stdout),
            String::from_utf8_lossy(&run_result.stderr)
        ));

        if !run_result.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to run scheduled task: {}",
                        String::from_utf8_lossy(&run_result.stderr))
            ));
        }

        // Rimuovi il task dopo l'avvio
        // log_message("Rimozione del task...");
        // let delete_result = Command::new("schtasks")
        //     .args([
        //         "/Delete",
        //         "/TN", "BackupProgramLauncher",
        //         "/F"
        //     ])
        //     .output()?;
        //
        // log_message(&format!(
        //     "Risultato rimozione task:\nStatus: {}\nStdout: {}\nStderr: {}",
        //     delete_result.status,
        //     String::from_utf8_lossy(&delete_result.stdout),
        //     String::from_utf8_lossy(&delete_result.stderr)
        // ));

        log_message("launch_backup_program completato");
        Ok(())
    }

    fn log_message(message: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("C:\\Temp\\backup_service_detailed.log")
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            if let Err(e) = writeln!(file, "[{}] {}", timestamp, message) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }
}

