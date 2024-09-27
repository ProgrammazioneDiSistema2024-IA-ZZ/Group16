use std::{fs, process, thread};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::Pid;

mod cpu_evaluation;
mod mouse_tracker;
mod audio;
mod display_window;
mod backup;

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

#[derive(Debug, serde::Deserialize)]
struct Config {
    source_path: String,
    destination_path: String,
    backup_type: String,
    extensions_to_backup: Vec<String>,
}

fn read_config(config_path: &str) -> Config {
    let mut file = fs::File::open(config_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    toml::from_str(&contents).unwrap()
}

fn main() {
    /* Get the monitor resolution */
    let (width, height) = get_screen_resolution();
    println!("Risoluzione dello schermo: {}, {}", width, height);

    /* Mouse Tracker enable */

    let window_enable = Arc::new(Mutex::new(false));
    let window_enable_clone = Arc::clone(&window_enable);

    mouse_tracker::track_mouse(window_enable_clone.clone(), width as f64, height as f64);


    let pid = Pid::from_u32(process::id());
    println!("Pid: {}", pid);

    cpu_evaluation::cpu_monitor(pid);

    loop {
        thread::sleep(Duration::from_secs(1));
        let mut enable = window_enable_clone.lock().unwrap();
        if *enable{
            *enable = false;
            if let Err(e) = display_window::show_gui(){
                eprintln!("Errore nella generazione della GUI: {}", e );
            }
        }
    }
}

