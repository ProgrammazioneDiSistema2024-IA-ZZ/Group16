use std::{fs, io};
use std::path::Path;
use fs_extra::dir::CopyOptions;
use fs_extra::dir;
use fs_extra::file;
use fs_extra::error::Error as FsExtraError;
use crate::Config;

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
        "full_disk" | "directory" => {
            dir::copy(source_path, destination_path, &dir_options)?;
        },
        "selective" => {
            for entry in fs::read_dir(source_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if config.extensions_to_backup.contains(&extension.to_string_lossy().into_owned()) {
                            let dest = destination_path.join(path.file_name().unwrap());
                            file::copy(path, dest, &file_options)?;
                        }
                    }
                }
            }
        },
        _ => return Err(BackupError::InvalidBackupType),
    }

    Ok(())
}