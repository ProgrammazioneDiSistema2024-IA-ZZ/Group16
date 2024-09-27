use sysinfo::{System, Pid};
use std::{fs::OpenOptions, thread};
use chrono::Local;
use std::io::Write;
use std::time::Duration;

pub fn create_file() -> std::fs::File {
        OpenOptions::new()
        .create(true)
        .append(true)
        .open("cpu_log.txt")
        .unwrap()
}

pub fn process_cpu_consumption(pid: Pid, cpu_log_file: &mut std::fs::File) {
    // Create a system object
    let mut sys = System::new_all();


    // Refresh the CPU consumption
    sys.refresh_cpu_usage();

    if let Some(process) = sys.process(pid) {
        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("{} - CPU Usage: {}%", current_time, process.cpu_usage());
        writeln!(cpu_log_file, "{}", log_entry).unwrap();
    }
}

pub fn cpu_monitor(pid: Pid) {
    let mut cpu_log_file = create_file();
    let monitor = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            process_cpu_consumption(pid, &mut cpu_log_file);
        }
    });

    monitor.join().expect("AIUTO");
}