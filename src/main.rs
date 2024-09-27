mod cpu_evaluation;
use std::process;
use sysinfo::Pid;
use std::{thread, time};

use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::{System, SystemExt};
use crate::display_window::show_gui;

#[cfg(target_os = "windows")]
fn get_screen_resolution() -> (i32, i32){
    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
    use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN};

    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN)};
    (width, height)
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

fn print_system_info(){
    let mut system = System::new_all();
    system.refresh_all();

    println!("Nome del sistema operativo: {:?}", system.name());
    println!("RAM totale: {} MB", system.total_memory());
    println!("Numero di cpu: {}", system.cpus().len());
}

mod mouse_tracker;
mod audio;
mod display_window;

fn main() {

    /**/

    /* let mut cpu_log_file = cpu_evaluation::create_file();
    let pid = Pid::from_u32(process::id());

    loop {
        cpu_evaluation::process_cpu_consumption(pid, &mut cpu_log_file);

        //Sleep for 2 minutes
        thread::sleep(time::Duration::from_secs(120));
    }*/

    let (width, height) = get_screen_resolution();
    println!("Risoluzione dello schermo: {}, {}", width, height);
    print_system_info();

    let window_enable = Arc::new(Mutex::new(false));
    let window_enable_clone = Arc::clone(&window_enable);

    mouse_tracker::track_mouse(window_enable_clone.clone(), width as f64, height as f64);

    loop {

        thread::sleep(Duration::from_secs(1));
        let mut enable = window_enable_clone.lock().unwrap();
        if *enable{
            *enable = false;
            if let Err(e) = show_gui(){
                eprintln!("Errore nella generazione della GUI: {}", e );
            }
        }
    }
}

