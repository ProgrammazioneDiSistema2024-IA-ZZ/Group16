use std::{io, process::Command};
use service_manager::*;
use std::ffi::OsString;
use std::path::{PathBuf};
use std::env;
use std::error::Error;

mod cpu_evaluation;
mod uninstall_service;
mod service;

#[cfg(target_os = "windows")]
fn main() -> windows_service::Result<()> {
    use std::ffi::OsString;
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // Installs the service defined in `service.rs`.
    let service_binary_path = ::std::env::current_exe().unwrap().with_file_name("service.exe");

    let service_info = ServiceInfo {
        name: OsString::from("BackMeUp"),
        display_name: OsString::from("BackMeUp"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description("Group16 BackMeUp Application Service")?;
    Ok(())
}


#[cfg(not(target_os = "windows"))]
fn is_service_installed(service_name: &str) -> Result<bool, Box<dyn Error>> {
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

#[cfg(not(target_os = "windows"))]
fn main() -> io::Result<()> {
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
                println!("Service already installed and running");
            } else {
                println!("Service not installed, starting installation..");
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

            }

            println!("Service started successfully");
        }
        Err(e) => {
            eprintln!("Error during service check: {}", e);
        }
    }

    Ok(())
}