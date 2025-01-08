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
    let destination_path = Path::new(&config.destination_path);

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
            let directory_size = calculate_directory_size(source_path)?;

            dir::copy(source_path, destination_path, &dir_options)?;

            // Update total size
            total_size += directory_size;
        },
        "selective" => {
            for entry in fs::read_dir(source_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if config.extensions_to_backup.contains(&extension.to_string_lossy().into_owned()) {
                            let dest = destination_path.join(path.file_name().unwrap());

                            // Calculate size of file
                            let file_size = fs::metadata(&path)?;
                            // Update total size
                            total_size += file_size.len();

                            file::copy(path, dest, &file_options)?;
                        }
                    }
                }
            }
        },
        _ => return Err(BackupError::InvalidBackupType),
    }

    let backup_time = start_time.elapsed();
    backup_monitor(destination_path, total_size, backup_time);
    Ok(())
}

fn calculate_directory_size(path: &Path) -> Result<u64, BackupError> {
    let mut total_size: u64 = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        // Check if entry is a file
        if metadata.is_file() {
            total_size += metadata.len();
        // Check if entry is a directory
        } else if metadata.is_dir() {
            // Recursively calculate size of directory with the same function
            total_size += calculate_directory_size(&entry.path())?;
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