use rdev::{listen, Event, EventType};
use std::sync::{Arc, Mutex};
use std::{env, fs, thread};
use std::path::PathBuf;
use std::process::Command;
use crate::audio::play_sound;
use crate::{backup};
#[derive(Debug, Clone)]
struct Point{
    x: f64,
    y: f64,
}

#[derive(Debug, PartialEq)]
enum Action {
    Background,
    Confirm,
    Cancel,
    Modify,
}

/// Funzione per controllare se un punto è vicino a un altro punto, entro una certa tolleranza
fn is_near(p1: &Point, p2: &Point, tolerance: f64) -> Action {
    if distance(p1, p2) <= tolerance {
        Action::Confirm
    } else {
        Action::Background
    }
}

fn is_border (p1: &Point, p2: &Point, direction: &str) -> bool {
    match direction {
        "horizontal" => (p1.y - p2.y).abs() < 30.0,  // Movimento orizzontale
        "vertical" => (p1.x - p2.x).abs() < 30.0,    // Movimento verticale
        "diagonal" => {
            let delta_x = p2.x - p1.x;
            let delta_y = p2.y - p1.y;

            // Verifica se il movimento è diagonale (sia x che y devono variare)
            (delta_x.abs() > 30.0 && delta_y.abs() > 30.0) && (delta_x / delta_y).abs() >= 1.0
        },
        _ => false,
    }
}

/// Funzione per controllare se il vettore di punti contiene i punti agli angoli dello schermo
fn contains_corners(
    points: &Vec<Point>,
    screen_width: f64,
    screen_height: f64,
    enable: bool
) -> Action {
    let tolerance = 50.0; // Definisci una tolleranza per la vicinanza agli angoli
    if !enable {
        let top_left = Point { x: 0.00, y: 0.00 };
        let top_right = Point { x: screen_width, y: 0.00 };
        let bottom_left = Point { x: 0.00, y: screen_height };
        let bottom_right = Point { x: screen_width, y: screen_height };

        let mut found_top_left = false;
        let mut found_top_right = false;
        let mut found_bottom_left = false;
        let mut found_bottom_right = false;

        let mut previous_point = None;


        for point in points {
            if is_near(&point, &top_left, tolerance) == Action::Confirm{
                found_top_left = true;
                previous_point = Some(point.clone());
            }
            if is_near(&point, &top_right, tolerance) == Action::Confirm{
                if let Some(prev) = &previous_point {
                    if found_top_left && is_border(prev, point, "horizontal") {
                        found_top_right = true;
                        previous_point = Some(point.clone());
                    } else {
                        found_top_left = false;
                        found_top_right = false;
                        found_bottom_left = false;
                        found_bottom_right = false;
                }}
            }
            if is_near(&point, &bottom_right, tolerance) == Action::Confirm{
                if let Some(prev) = &previous_point{
                    if found_top_right && is_border(prev, point, "vertical"){
                        found_bottom_right = true;
                        previous_point = Some(point.clone());
                    } else {
                        found_top_left = false;
                        found_top_right = false;
                        found_bottom_left = false;
                        found_bottom_right = false;
                }}
            }
            if is_near(&point, &bottom_left, tolerance) == Action::Confirm{
                if let Some(prev) = &previous_point{
                    if found_bottom_right && is_border(prev, point, "horizontal") {
                        found_bottom_left = true;
                    } else {
                        found_top_left = false;
                        found_top_right = false;
                        found_bottom_left = false;
                        found_bottom_right = false;
                    }}
            }
        }
        if found_top_left && found_top_right && found_bottom_left && found_bottom_right {
            Action::Confirm
        } else {
            Action::Background
        }
    } else {
        // Definisci gli angoli necessari
        let bottom_left = Point { x: 0.00, y: screen_height };
        let bottom_right = Point { x: screen_width, y: screen_height };
        let top_left = Point { x: 0.00, y: 0.00 };
        let top_right = Point { x: screen_width, y: 0.00 };

        let mut passed_bottom_left = false;
        let mut previous_point = None;

        for point in points {
            // Se il mouse è passato dall'angolo in basso a sinistra
            if is_near(&point, &bottom_left, tolerance) == Action::Confirm {
                passed_bottom_left = true;
                previous_point = Some(point.clone());
            }

            if passed_bottom_left {
                // Se il mouse va dall'angolo in basso a sinistra a quello in basso a destra, confermo il backup
                if is_near(&point, &bottom_right, tolerance) == Action::Confirm {
                    if let Some(prev) = &previous_point {
                        if is_border(prev, point, "horizontal") {
                            return Action::Confirm;
                        }
                    }
                }
                // Se il mouse va dall'angolo in basso a sinistra a quello in alto a sinistra, annullo il backup
                if is_near(&point, &top_left, tolerance) == Action::Confirm {
                    if let Some(prev) = &previous_point {
                        if is_border(prev, point, "vertical") {
                            return Action::Cancel;
                        }
                    }
                }
                // Se il mouse va dall'angolo in basso a sinistra a quello in alto a destra, ritorna false
                if is_near(&point, &top_right, tolerance) == Action::Confirm {
                    if let Some(prev) = &previous_point {
                        if is_border(prev, point, "diagonal") {
                            return Action::Modify;
                        }
                    }
                }
            }
        }

        // Se non è stato trovato nessun percorso specifico, ritorna false di default
        Action::Background
    }
}

/// Funzione per calcolare la distanza tra due punti
fn distance(p1: &Point, p2: &Point) -> f64 {
    ((p1.x - p2.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}

pub fn track_mouse(screen_width: f64, screen_height: f64) {
    println!("Tracciamento abilitato!");

    let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
    let audio_path: PathBuf = exe_path.parent().unwrap().join("Resources/audio/");
    let config_file_path: PathBuf = exe_path.parent().unwrap().join("Resources/");


    let points = Arc::new(Mutex::new(Vec::<Point>::new()));
    let points_clone = Arc::clone(&points);
    let tracking_enabled = Arc::new(Mutex::new(false));
    let tracking_enabled_clone = Arc::clone(&tracking_enabled);


    thread::spawn(move || {
        listen(move |event: Event| {
            if let EventType::MouseMove { x, y } = event.event_type {
                let point = Point { x, y };

                // Controlla se il tracciamento è abilitato
                let enabled = *tracking_enabled_clone.lock().unwrap();

                let mut points = points_clone.lock().unwrap();
                points.push(point.clone());

                println!("Punto tracciato: ({:.2}, {:.2})", point.x, point.y);

                // Verifica se ci sono abbastanza punti per riconoscere gli angoli dello schermo
                if !enabled && contains_corners(&points, screen_width, screen_height, enabled) == Action::Confirm {
                    let mut enabled_ref = tracking_enabled_clone.lock().unwrap();
                    *enabled_ref = true;  // Cambia qui lo stato di tracking_enabled
                    play_sound(0);
                    points.clear();

                    if let Err(e) = Command::new(exe_path.join("config_program")).arg("backup").spawn() {
                        eprintln!("Failed to spawn process: {}", e);
                    }

                }

                if enabled && contains_corners(&points, screen_width, screen_height, enabled) == Action::Modify {
                    points.clear();
                    let mut enabled_ref = tracking_enabled_clone.lock().unwrap();
                    *enabled_ref = false;  // Cambia qui lo stato di tracking_enabled


                    if config_file_path.join("config.toml").exists(){
                        fs::remove_file(config_file_path.join("config.toml")).expect("Error deleting file");
                    }

                    if let Err(e) = Command::new(exe_path.join("config_program")).arg("config").spawn() {
                        eprintln!("Failed to spawn process: {}", e);
                    }
                }

                // Se il tracciamento è abilitato, verifica se viene disegnato un "+", e non solo gli angoli
                if enabled && contains_corners(&points, screen_width, screen_height, enabled) == Action::Confirm {

                    if config_file_path.join("config.toml").exists() {
                        let config = backup::read_config(config_file_path.join("config.toml").to_str().unwrap());

                        // faccio il backup
                        match backup::backup_files(&config) {
                            Ok(_) => println!("Backup completed successfully"),
                            Err(e) => match e {
                                backup::BackupError::SourceNotFound =>
                                    eprintln!("Backup failed: Source path does not exist"),
                                backup::BackupError::InvalidBackupType =>
                                    eprintln!("Backup failed: Invalid backup type specified"),
                                backup::BackupError::IoError(e) =>
                                    eprintln!("Backup failed due to IO error: {}", e),
                                backup::BackupError::FsExtraError(e) =>
                                    eprintln!("Backup failed due to fs_extra error: {}", e),
                            }
                        }

                        // let mut enabled_ref = tracking_enabled_clone.lock().unwrap();
                        // *enabled_ref = false;  // Cambia qui lo stato di tracking_enabled
                        play_sound(1);
                        // points.clear();
                    }else{
                        play_sound(2);
                        eprintln!("File di configurazione non trovato! Backup non eseguito.");

                    }
                    let mut enabled_ref = tracking_enabled_clone.lock().unwrap();
                    *enabled_ref = false;  // Cambia qui lo stato di tracking_enabled
                    points.clear();
                }
                if enabled && contains_corners(&points, screen_width, screen_height, enabled) == Action::Cancel {

                    println!("Backup cancelled");
                    play_sound(2);
                    points.clear();
                    let mut enabled_ref = tracking_enabled_clone.lock().unwrap();
                    *enabled_ref = false;  // Cambia qui lo stato di tracking_enabled
                }
            }
        }).unwrap();
    });
}
