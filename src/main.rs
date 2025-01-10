use std::{io, process::Command};
use sysinfo::Pid;
use service_manager::*;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::env;
use std::error::Error;
use std::process::Stdio;

mod cpu_evaluation;


fn is_service_installed(service_name: &str) -> Result<bool, Box<dyn Error>> {
    #[cfg(target_os = "windows")]
    {
        // Controlla se il servizio è installato su Windows
        let output = Command::new("sc")
            .arg("query")
            .arg(service_name)
            .output()?;

        // Se il codice di uscita è 0, il servizio esiste
        Ok(output.status.success())
    }

    #[cfg(target_os = "macos")]
    {
        // Controlla se il servizio è installato su macOS
        let output = Command::new("launchctl")
            .arg("list")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Verifica se il nome del servizio è presente nell'output
        Ok(stdout.contains(service_name))
    }

    #[cfg(target_os = "linux")]
    {
        // Controlla se il servizio è installato su Linux
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg(service_name)
            .output()?;

        // Se il codice di uscita è 0, il servizio esiste
        Ok(output.status.success())
    }
}

fn make_automated_service() {
    // Create a label for our service
    let label: ServiceLabel = "com.backmeup".parse().unwrap();

    // Get generic service by detecting what is available on the platform
    let mut manager = <dyn ServiceManager>::native()
        .expect("Failed to detect management platform");

    // Update our manager to work with user-level services
    manager.set_level(ServiceLevel::User)
        .expect("Service manager does not support user-level services");


    match is_service_installed("com.backmeup") {
        Ok(installed) => {
            if installed {
                // println!("Service already installed and running");
            } else {
                // println!("Service not installed, starting installation..");
                // Install our service using the underlying service management platform
                manager.install(ServiceInstallCtx {
                    label: label.clone(),
                    // program: PathBuf::from(env::current_exe().unwrap().join("/Group16")),
                    program: PathBuf::from(env::current_exe().unwrap().parent().unwrap().join("backup_program")),
                    args: vec![OsString::from("--service")],
                    contents: None, // Optional String for system-specific service content.
                    username: None, // Optional String for alternative user to run service.
                    working_directory: None, // Optional String for the working directory for the service process.
                    environment: None, // Optional list of environment variables to supply the service process.
                    autostart: true, // Specify whether the service should automatically start upon OS reboot.
                }).expect("Failed to install");

                // // Start our service using the underlying service management platform
                // manager.start(ServiceStartCtx {
                //     label: label.clone()
                // }).expect("Failed to start");
            }

            // println!("Service started successfully");
        }
        Err(e) => {
            // eprintln!("Error during service check: {}", e);
        }
    }

}

fn main() -> io::Result<()> {
    // println!("Avvio del programma di config...");
    // println!("Percorso: {:?}", env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("assets/blip-131856.mp3"));

    // Sto iniziando a svarionare: stavo pensando se ha senso eliminare da qui il lancio di config_program dato che ora è incluso in backup_program qualora config.toml non esista
    // A questo punto questo programma si occuperebbe solo di installare il servizio..


    let exe_path: PathBuf = PathBuf::from(env::current_exe()?.parent().unwrap());
    let config_program_path = exe_path.join("config_program");
    let backup_program_path = exe_path.join("backup_program");
    let config_file_path = exe_path.parent().unwrap().join("Resources/");

    // make_automated_service();

    // Start the config_program and capture its PID
    // let config_program = Command::new(config_program_path).arg("config").output()?;  // `spawn` instead of `output` to get the PID

    // If config_program was successful, start backup_program
    // if config_program.status.success() && config_file_path.join("config.toml").exists() {
        make_automated_service();

        // let backup_program = Command::new(backup_program_path).spawn()?;
        //
        // let backup_pid = Pid::from_u32(backup_program.id());
        // cpu_evaluation::start_cpu_monitor(backup_pid, 30);
        // backup_program.wait_with_output().expect("TODO: panic message");
    // } else {
    //     eprintln!("Errore");
    //     eprintln!("Errore: {:?}", config_program);
    // }

    Ok(())
}

fn run_service() -> io::Result<()>{
    // Logica del servizio qui
    // Ad esempio, avviare backup_program e monitorare la CPU
    println!("Avvio del programma di backup... {:?}", PathBuf::from(env::current_exe()?.parent().unwrap().join("backup_program")));
    let backup_program = Command::new(PathBuf::from(env::current_exe()?.parent().unwrap().join("backup_program")))
        .spawn()?;

    let backup_pid = Pid::from_u32(backup_program.id());
    cpu_evaluation::start_cpu_monitor(backup_pid, 30);
    backup_program.wait_with_output().expect("TODO: panic message");

    Ok(())
}

fn run_config_program() -> io::Result<()>{
    // Logica per l'installazione e la configurazione del servizio
    make_automated_service();

    // Avvia il servizio se necessario
    // let manager = <dyn ServiceManager>::native().expect("Failed to detect management platform");
    // manager.start(ServiceStartCtx {
    //     label: "com.backmeup".parse().unwrap()
    // }).expect("Failed to start service");

    // Start the config_program and capture its PID
    // println!("Avvio del programma di config...");
    // let config_program = Command::new(PathBuf::from(env::current_exe()?.parent().unwrap().join("config_program")))
    //     .output()?;  // `spawn` instead of `output` to get the PID
    //
    // // If config_program was successful, start backup_program
    // if config_program.status.success() {
    //     println!("Avvio del programma di backup...");
    //     let backup_program = Command::new(PathBuf::from(env::current_exe()?.parent().unwrap().join("backup_program")))
    //         .spawn()?;
    //
    //     let backup_pid = Pid::from_u32(backup_program.id());
    //     cpu_evaluation::start_cpu_monitor(backup_pid, 30);
    //     backup_program.wait_with_output().expect("TODO: panic message");
    // } else {
    //     eprintln!("Errore");
    //     eprintln!("Errore: {:?}", config_program);
    // }

    Ok(())
}

// fn main() -> io::Result<()> {
//     let args: Vec<String> = env::args().collect();
//
//     if args.len() > 1 && args[1] == "--service" {
//         run_service().expect("TODO: panic message");
//     } else {
//         run_config_program().expect("TODO: panic message");
//     }
//
//     Ok(())
// }
