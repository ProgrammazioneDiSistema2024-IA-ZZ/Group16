use std::{io, process::Command};
use sysinfo::Pid;

mod cpu_evaluation;

fn main() -> io::Result<()> {
    println!("Avvio del programma di config...");

    // Start the config_program and capture its PID
    let config_program = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("config_program")
        .output()?;  // `spawn` instead of `output` to get the PID

    // If config_program was successful, start backup_program
    if config_program.status.success() {
        let backup_program = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("backup_program")
            .spawn()?;

        let backup_pid = Pid::from_u32(backup_program.id());
        cpu_evaluation::start_cpu_monitor(backup_pid, 30);
        backup_program.wait_with_output().expect("TODO: panic message");
    } else {
        eprintln!("Errore");
    }

    Ok(())
}
