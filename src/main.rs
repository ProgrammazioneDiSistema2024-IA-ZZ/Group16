mod cpu_evaluation;
use std::process;
use sysinfo::Pid;
use std::{thread, time};

fn main() {

    println!("Hello, world!");

    let mut cpu_log_file = cpu_evaluation::create_file();
    let pid = Pid::from_u32(process::id());

    loop {
        cpu_evaluation::process_cpu_consumption(pid, &mut cpu_log_file);

        //Sleep for 2 minutes
        thread::sleep(time::Duration::from_secs(120));
    }

}

