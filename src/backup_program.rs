mod cpu_evaluation;
mod mouse_tracker;
mod audio;
mod backup;
mod display_window;

#[cfg(target_os = "windows")]
fn get_screen_resolution() -> (usize, usize){

    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
    use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN};

    let mut width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    width = width + 25*width/100;
    let mut height = unsafe { GetSystemMetrics(SM_CYSCREEN)};
    height = height + 25*height/100;
    (width as usize, height as usize)
}

#[cfg(target_os = "macos")]
fn get_screen_resolution() -> (usize, usize){
    use core_graphics::display::CGDisplay;

    let display = CGDisplay::main();
    (display.pixels_wide() as usize, display.pixels_high() as usize)
}

#[cfg(target_os = "linux")]
fn get_screen_resolution() -> (u32, u32){
    use x11::xlib;
    use std::ptr;

    unsafe{
        let display = xlib::XOpenDisplay(ptr::null());
        let screen = xlib::XDefaultScreen(display);
        let width = xlib::XDisplayWidth(display, screen) as u32;
        let height = xlib::XDisplayHeight(display, screen) as u32;
        xlib::XCloseDisplay(display);
        (width, height)
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
    let config_program_path = exe_path.join("config_program");
    let config_file_path = exe_path.parent().unwrap().join("Resources/");

    println!("Percorso del file di configurazione: {:?}", config_file_path.join("config.toml"));

    // Check if config.toml exists.
    // If not, start the config program. This is done in case system is rebooted, backup_program service is started but the config.toml is deleted.
    if !config_file_path.join("config.toml").exists() {
        Command::new(config_program_path).arg("config").spawn().expect("Failed to start config program");
    }

    /* Start the actual backup program */

    // Get the monitor resolution
    let (width, height) = get_screen_resolution();
    println!("Risoluzione dello schermo: {}, {}", width, height);

    mouse_tracker::track_mouse(width as f64, height as f64);

    let backup_pid = Pid::from_u32(process::id());
    cpu_evaluation::start_cpu_monitor(backup_pid, 120);

    // Loop to keep the program alive
    loop {
        thread::sleep(Duration::from_secs(1)); // faccio un ciclo al secondo
    }

}

#[cfg(target_os = "windows")]
fn main() -> windows_service::Result<()> {
    backup_windows_service::run()
}

#[cfg(target_os = "windows")]
mod backup_windows_service {
    use std::{env, ffi::OsString, sync::mpsc, time::Duration};
    use std::path::PathBuf;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };
    use std::{process};
    use std::process::Command;
    use sysinfo::Pid;
    use crate::cpu_evaluation;
    use crate::mouse_tracker;
    use crate::get_screen_resolution;

    const SERVICE_NAME: &str = "backup_program";
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

        let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
        let config_program_path = exe_path.join("config_program");
        let config_file_path = exe_path.parent().unwrap().join("Resources/");

        println!("Percorso del file di configurazione: {:?}", config_file_path.join("config.toml"));

        // Check if config.toml exists.
        // If not, start the config program. This is done in case system is rebooted, backup_program service is started but the config.toml is deleted.
        if !config_file_path.join("config.toml").exists() {
            Command::new(config_program_path).arg("config").spawn().expect("Failed to start config program");
        }

        /* Start the actual backup program */

        // Get the monitor resolution
        let (width, height) = get_screen_resolution();
        println!("Risoluzione dello schermo: {}, {}", width, height);

        mouse_tracker::track_mouse(width as f64, height as f64);

        let backup_pid = Pid::from_u32(process::id());
        cpu_evaluation::start_cpu_monitor(backup_pid, 120);

        loop {
            // Poll shutdown event.
            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                // Break the loop either upon stop or channel disconnect
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                // Continue work if no events were received within the timeout
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            };
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
}