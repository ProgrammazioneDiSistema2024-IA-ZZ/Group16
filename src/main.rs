use std::{io, process::Command};

fn main() -> io::Result<()> {
    println!("Avvio del programma di config...");

    // Start the config_program and capture its PID
    let config_program = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("config_program")
        .arg("config")
        .output()?;  // `spawn` instead of `output` to get the PID

    // If config_program was successful, start backup_program
    if config_program.status.success() {
        let backup_program = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("backup_program")
            .spawn()?;
        backup_program.wait_with_output().expect("TODO: panic message");
    } else {
        eprintln!("Errore");
    }

    Ok(())
}
