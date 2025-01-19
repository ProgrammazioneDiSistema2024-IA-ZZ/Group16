use std::{fs, io};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use fs_extra::dir::CopyOptions;
use fs_extra::dir;
use fs_extra::file;
use fs_extra::error::Error as FsExtraError;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub source_path: String,
    pub destination_path: String,
    pub backup_type: String,
    pub extensions_to_backup: Vec<String>,
}

pub fn read_config(config_path: &str) -> Config {
    let mut file = fs::File::open(config_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    toml::from_str(&contents).unwrap()
}

#[derive(Debug)]
pub(crate) enum BackupError {
    SourceNotFound,
    InvalidBackupType,
    IoError(io::Error),
    FsExtraError(FsExtraError),
}

impl From<io::Error> for BackupError {
    fn from(error: io::Error) -> Self {
        BackupError::IoError(error)
    }
}

impl From<FsExtraError> for BackupError {
    fn from(error: FsExtraError) -> Self {
        BackupError::FsExtraError(error)
    }
}

pub(crate) fn backup_files(config: &Config) -> Result<(), BackupError> {
    let source_path = Path::new(&config.source_path);
    let mut destination_path = Path::new(&config.destination_path);

    // Copy last folder name in source_path into destination_path
    let source_folder_name = source_path.file_name().unwrap();
    let destination_path = &*destination_path.join(source_folder_name);

    println!("Backup started from: {:?}", source_path);
    println!("Backup towards folder: {:?}", destination_path);

    // Total size of files copied
    let mut total_size: u64 = 0;

    let start_time = Instant::now();

    // Check if source path exists
    if !source_path.exists() {
        return Err(BackupError::SourceNotFound);
    }

    // Create destination directory if it doesn't exist
    if !destination_path.exists() {
        fs::create_dir_all(destination_path)?;
        println!("Created destination directory: {:?}", destination_path);
    }

    let mut dir_options = CopyOptions::new();
    dir_options.overwrite = true;

    let file_options = file::CopyOptions::new();

    match config.backup_type.as_str() {
        "full-disk" | "directory" => {
            // Calculate size of directory
            total_size = calculate_directory_size(source_path)?;

            // Copy directory recursively
            copy_recursive(source_path, destination_path, None)?;
        },
        "selective" => {
            // Copy files with specific extensions
            total_size = copy_recursive(source_path, destination_path, Some(&config.extensions_to_backup))?;
        },
        _ => return Err(BackupError::InvalidBackupType),
    }

    let backup_time = start_time.elapsed();
    backup_monitor(destination_path, total_size, backup_time);
    Ok(())
}

fn calculate_directory_size(path: &Path) -> Result<u64, BackupError> {
    let mut total_size: u64 = 0;

    println!("Inizio elaborazione: {}", path.display());

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Errore nell'accesso alla directory {}: {}. Ignorata.", path.display(), e);
            return Ok(0); // Restituisci una dimensione di 0 per directory non accessibili
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Errore nell'elaborazione di un elemento in {}: {}. Ignorato.", path.display(), e);
                continue; // Salta l'elemento e passa al successivo
            }
        };

        println!("Elaborando: {:?}", entry.path());

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Errore nel leggere i metadata di {:?}: {}. Ignorato.", entry.path(), e);
                continue; // Salta l'elemento e passa al successivo
            }
        };

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            match calculate_directory_size(&entry.path()) {
                Ok(size) => total_size += size,
                Err(e) => {
                    eprintln!("Errore nella calcolazione della dimensione di {}: {:?}. Ignorato.", entry.path().display(), e);
                    continue; // Ignora la directory annidata e continua
                }
            }
        }
    }

    Ok(total_size)
}

fn backup_monitor(destination_path: &Path, total_size: u64, backup_time: Duration) {
    let log_path = destination_path.join("backup_log.txt");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();

    writeln!(file, "Total size of saved files: {} bytes", total_size).unwrap();
    writeln!(file, "Backup completed in: {:.2} seconds", backup_time.as_secs_f64()).unwrap();
}

fn copy_recursive<P: AsRef<Path>, Q: AsRef<Path>>(
    from: P,
    to: Q,
    extensions_to_backup: Option<&Vec<String>>,
) -> io::Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    if !from.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Percorso non trovato: {}", from.display()),
        ));
    }

    if !from.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Il percorso non è una directory: {}", from.display()),
        ));
    }

    let mut total_size = 0;

    for entry in fs::read_dir(from)? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Errore durante l'accesso a un elemento in {}: {}. Ignorato.", from.display(), e);
                continue;
            }
        };

        let entry_path = entry.path();
        let dest_path = to.join(entry.file_name());

        if entry_path.is_dir() {
            // Crea la directory di destinazione e copia ricorsivamente
            if let Err(e) = fs::create_dir_all(&dest_path) {
                eprintln!("Errore durante la creazione della directory {}: {}. Ignorata.", dest_path.display(), e);
                continue;
            }
            match copy_recursive(&entry_path, &dest_path, extensions_to_backup) {
                Ok(size) => total_size += size,
                Err(e) => eprintln!("Errore durante la copia della directory {}: {}. Ignorata.", entry_path.display(), e),
            }
        } else if entry_path.is_file() {
            // Se ci sono estensioni specificate, copia solo i file con le estensioni corrispondenti
            if let Some(extensions) = extensions_to_backup {
                if let Some(extension) = entry_path.extension() {
                    if !extensions.contains(&extension.to_string_lossy().to_string()) {
                        continue; // Salta i file che non corrispondono
                    }
                } else {
                    continue; // Salta i file senza estensione
                }
            }

            // Copia il file e aggiungi la dimensione al totale
            match fs::copy(&entry_path, &dest_path) {
                Ok(size) => total_size += size,
                Err(e) => eprintln!("Errore durante la copia del file {}: {}. Ignorato.", entry_path.display(), e),
            }
        }
    }

    Ok(total_size)
}
