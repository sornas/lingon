//! The asset subsystem.
//!
//! This handles loading and reloading of asset files. All loads return an ID; for example,
//! [AssetSystem::load_audio] returns an [AudioAssetID]. To get the actual asset, index the asset
//! system like this:
//!
//! ```ignored
//! let assets = AssetSystem::new();
//! let audio_id = assets.load_audio(Path::new("path/to/sound.wav".to_path_buf()));
//! let audio_asset = assets[audio_id];
//! ```
//!
//! When building with `cfg(debug_assertions)` (i.e. without `--release`) assets are hot-reloaded.

pub mod audio;
pub mod image;

pub use audio::Audio;
pub use image::Image;

use std::ops::Index;
use std::path::PathBuf;
use std::time::SystemTime;

/// A marker type for the unit pixels.
pub type Pixels = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageAssetID(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioAssetID(pub usize);

/// If the type of asset type is unknown or doesn't matter.
pub enum AssetID {
    Image(ImageAssetID),
    Audio(AudioAssetID),
}

pub struct AssetSystem {
    images: Vec<Image>,
    audio: Vec<Audio>,
}

impl AssetSystem {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            audio: Vec::new(),
        }
    }

    /// Load a new image from disk.
    pub fn load_image(&mut self, file: PathBuf) -> ImageAssetID {
        let id = self.images.len();
        self.images.push(Image::new(file));
        ImageAssetID(id)
    }

    /// Load a new sound from disk.
    pub fn load_audio(&mut self, file: PathBuf) -> AudioAssetID {
        let id = self.audio.len();
        self.audio.push(Audio::new(file));
        AudioAssetID(id)
    }

    pub fn reload(&mut self) {
        for audio in self.audio.iter_mut() {
            audio.reload();
        }
    }
}

impl Index<ImageAssetID> for AssetSystem {
    type Output = Image;

    fn index(&self, id: ImageAssetID) -> &Self::Output {
        self.images.get(id.0).expect(&format!("Invalid image asset {}", id.0))
    }
}

impl Index<AudioAssetID> for AssetSystem {
    type Output = Audio;

    fn index(&self, id: AudioAssetID) -> &Self::Output {
        self.audio.get(id.0).expect(&format!("Invalid audio asset {}", id.0))
    }
}

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
