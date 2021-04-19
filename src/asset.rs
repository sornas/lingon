mod image;

pub use image::Image;

use std::path::PathBuf;
use std::time::SystemTime;

/// A marker type for the unit pixels.
pub type Pixels = usize;

/// A file on disk that has been loaded.
#[derive(Clone, Debug)]
pub struct LoadedFile {
    pub file: PathBuf,
    pub last_modified: SystemTime,
}

impl LoadedFile {
    pub fn new(file: PathBuf) -> (Self, Vec<u8>) {
        let last_modified = std::fs::metadata(&file)
            .expect(&format!("asset file {} not found", file.display()))
            .modified()
            .ok()
            .unwrap_or_else(SystemTime::now);
        let bytes =
            std::fs::read(&file).expect(&format!("asset file {} not found", file.display()));
        (
            Self {
                file,
                last_modified,
            },
            bytes,
        )
    }

    /// Return the file data if it has been modified since it was last read.
    ///
    /// Modification is checked using [std::fs::metadata] and as such might not work on all
    /// operating systems.
    pub fn reload(&mut self) -> Option<Vec<u8>> {
        if cfg!(debug_assertions) {
            match std::fs::metadata(&self.file)
                .ok()
                .map(|m| m.modified().ok())
                .flatten()
            {
                Some(last_modified) if last_modified != self.last_modified => {
                    let bytes = std::fs::read(&self.file).ok();
                    if bytes.is_some() {
                        self.last_modified = last_modified;
                    }
                    bytes
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
