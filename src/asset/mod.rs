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

macro_rules! impl_deref_and_from_usize {
    ( $( $asset_type:ident ),* $(,)? ) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $asset_type(usize);

            impl ::std::ops::Deref for $asset_type {
                type Target = usize;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl $asset_type {
                /// Wrap a usize as an ID.
                ///
                /// # Safety
                ///
                /// The usize needs to be a valid ID that has previously been returned
                /// from the asset system.
                pub unsafe fn from_usize(u: usize) -> Self {
                    Self(u)
                }
            }
        )*
    }
}

impl_deref_and_from_usize!(
    ImageAssetID,
    AudioAssetID,
);

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
        // Image assets are reloaded by the renderer, which also uploads them.
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

// Number of frames to wait before reload.
const ASSET_COUNTDOWN: usize = 20;
#[derive(Clone, Debug)]
pub struct LoadedFile {
    pub file: PathBuf,
    pub countdown: usize,
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
                countdown: 0,
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
                    self.last_modified = last_modified;
                    self.countdown = 0;
                    None
                }
                Some(_) => {
                    if self.countdown < ASSET_COUNTDOWN {
                        self.countdown += 1;
                    }
                    if self.countdown == ASSET_COUNTDOWN {
                        let bytes = std::fs::read(&self.file).ok();
                        if bytes.is_some() {
                            self.countdown += 1;
                        }
                        bytes
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
