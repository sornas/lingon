use super::LoadedFile;

use std::convert::{TryFrom, TryInto};
use std::{ops::Deref, path::PathBuf};
use std::sync::{Arc, RwLock};

/// The different kinds of files we can open.
enum AudioFileKind {
    Ogg,
    Wav,
}

impl TryFrom<&str> for AudioFileKind {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "ogg" => Ok(AudioFileKind::Ogg),
            "wav" => Ok(AudioFileKind::Wav),
            _ => Err(()),
        }
    }
}

/// Actual audio data.
#[derive(Clone)]
pub struct Samples(Arc<RwLock<Vec<f32>>>);

impl Samples {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Vec::new())))
    }
}

impl Deref for Samples {
    type Target = Arc<RwLock<Vec<f32>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Audio {
    samples: Samples,
    data: LoadedFile,
    kind: AudioFileKind,
}

impl Audio {
    pub fn new(file: PathBuf) -> Option<Self> {
        let kind: AudioFileKind = file.extension()?.to_str()?.try_into().ok()?;
        let (data, bytes) = LoadedFile::new(file);
        let mut ret = Self {
            samples: Samples::new(),
            data,
            kind,
        };
        ret.load_data(bytes);
        Some(ret)
    }

    pub fn samples(&self) -> Samples {
        self.samples.clone()
    }

    pub fn reload(&mut self) -> bool {
        //TODO Repeated code here
        if let Some(bytes) = self.data.reload() {
            self.load_data(bytes);
            true
        } else {
            false
        }
    }

    pub fn load_data(&mut self, bytes: Vec<u8>) {
        match self.kind {
            AudioFileKind::Ogg => self.load_ogg(bytes),
            AudioFileKind::Wav => self.load_wav(bytes),
        }
    }

    pub fn load_wav(&mut self, bytes: Vec<u8>) {
        let (header, data) = wav::read(&mut std::io::Cursor::new(bytes)).unwrap();
        if header.sampling_rate as i32 != crate::audio::SAMPLE_RATE {
            println!(
                "warn: {} contains non-supported sample rate {}\nwarn: only 48000 is currently supported",
                 self.data.file.display(),
                 header.sampling_rate,
            );
        }
        let mut samples = self.samples.write().unwrap();
        match data {
            wav::BitDepth::ThirtyTwoFloat(data) => *samples = data,
            _ => todo!("Only WAV containing floats are currently supported"),
        }
    }

    pub fn load_ogg(&mut self, bytes: Vec<u8>) {
        todo!();
    }
}
