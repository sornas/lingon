mod audio;
mod image;

pub use audio::Audio;
pub use image::Image;

use std::ops::Index;
use std::path::PathBuf;
use std::time::SystemTime;

/// A marker type for the unit pixels.
pub type Pixels = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageAssetID(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioAssetID(usize);

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

/// Gets an asset from the asset system.
///
/// While this might look a bit bloated, it accomplishes two things.
///
/// 1) You can only get (e.g.) an [Image] from an [ImageAssetID].
/// 2) You can always `get` the asset, regardless of its type. Otherwise you'd have
/// `assets.image(image)`
// This might in the future be solvable with generic associated types. Something like:
//
// impl<'a> Index<ImageAssetID> for AssetSystem {
//     type Output = Option<&'a Image>;
// 
//     fn index(&'a self, id: ImageAssetID) -> &Self::Output {
//         self.images.get(id.0)
//     }
// }
pub trait TryGetAsset<O, I> {
    /// A reference to a loaded asset.
    fn try_get(&self, id: I) -> Option<&O>;
}

impl TryGetAsset<Image, ImageAssetID> for AssetSystem {
    fn try_get(&self, id: ImageAssetID) -> Option<&Image> {
        self.images.get(id.0)
    }
}

impl TryGetAsset<Audio, AudioAssetID> for AssetSystem {
    fn try_get(&self, id: AudioAssetID) -> Option<&Audio> {
        self.audio.get(id.0)
    }
}

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
