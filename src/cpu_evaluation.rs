use sysinfo::{System, Pid, ProcessesToUpdate};
use std::{fs::OpenOptions, thread};
use chrono::Local;
use std::io::Write;
use std::time::Duration;


/// Create or open a log file to store CPU usage data.
fn create_log_file() -> std::fs::File {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open("cpu_log.txt")
        .expect("Unable to create or open log file")
}

/// Log CPU usage for a given process using its PID.
fn log_cpu_usage(pid: Pid, log_file: &mut std::fs::File, sys: &mut System) {
    // Refresh processes to get up-to-date information
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]));

    if let Some(process) = sys.process(pid) {
        // Format current time and CPU usage
        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!(
            "{} - CPU Usage: {:.6}%",
            current_time,
            process.cpu_usage() / sys.cpus().len() as f32,
        );
        writeln!(log_file, "{}", log_entry).expect("Unable to write to log file");
    } else {
        // Log error if process with given PID is not found
        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("{} - Process with PID {} not found", current_time, pid);
        writeln!(log_file, "{}", log_entry).expect("Unable to write to log file");
    }
}

/// Start monitoring CPU usage for a specific process.
pub fn start_cpu_monitor(pid: Pid, interval_secs: u64) {
    let mut log_file = create_log_file(); // Open log file
    let mut sys = System::new_all();      // Create sysinfo system object

    // Spawn a thread for continuous monitoring
    thread::spawn(move || {
        loop {
            log_cpu_usage(pid, &mut log_file, &mut sys);
            thread::sleep(Duration::from_secs(interval_secs));
        }
    });
}