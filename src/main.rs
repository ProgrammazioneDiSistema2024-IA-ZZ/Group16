use std::{io};
use std::process::Command;

fn main() -> io::Result<()>{
    println!("Avvio del programma di config...");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("config_program")
        .output()?;

    if output.status.success(){
        Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("backup_program")
            .output()?;
    }else {
        eprintln!("Errore");
    }

    Ok(())
    /*

    */
}

