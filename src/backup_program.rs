use std::{process, thread};
use std::time::Duration;
use sysinfo::Pid;
use eframe::{egui, Frame};

mod cpu_evaluation;
mod mouse_tracker;
mod audio;
mod backup;
mod display_window;

#[cfg(target_os = "windows")]
fn get_screen_resolution() -> (usize, usize){

    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
    use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN};

    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN)};
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

fn main() {
    /* Get the monitor resolution */
    let (width, height) = get_screen_resolution();
    println!("Risoluzione dello schermo: {}, {}", width, height);

    mouse_tracker::track_mouse(width as f64, height as f64);

    let backup_pid = Pid::from_u32(process::id());
    cpu_evaluation::start_cpu_monitor(backup_pid, 30);

    // Loop to keep the program alive
    loop {
        thread::sleep(Duration::from_secs(1)); // faccio un ciclo al secondo
    }
}