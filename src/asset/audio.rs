use super::LoadedFile;

use std::convert::{TryFrom, TryInto};
use std::io::Cursor;
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
        let samples = match self.kind {
            AudioFileKind::Ogg => self.samples_ogg(bytes),
            AudioFileKind::Wav => self.samples_wav(bytes),
        };
        *self.samples.write().unwrap() = samples;
    }

    pub fn samples_wav(&mut self, bytes: Vec<u8>) -> Vec<f32> {
        let (header, data) = wav::read(&mut std::io::Cursor::new(bytes)).unwrap();
        if header.sampling_rate != crate::audio::SAMPLE_RATE as u32 {
            println!(
                "warn: {} contains non-supported sample rate {}\nwarn: only 48000 is currently supported",
                 self.data.file.display(),
                 header.sampling_rate,
            );
        }
        match data {
            wav::BitDepth::ThirtyTwoFloat(data) => data,
            _ => todo!("Only WAV containing floats are currently supported"),
        }
    }

    pub fn samples_ogg(&mut self, bytes: Vec<u8>) -> Vec<f32> {
        let mut reader = lewton::inside_ogg::OggStreamReader::new(Cursor::new(&bytes)).unwrap();
        let sample_rate = reader.ident_hdr.audio_sample_rate;
        if sample_rate != crate::audio::SAMPLE_RATE as u32 {
            println!(
                "warn: {} contains non-supported sample rate {}\nwarn: only 48000 is currently supported",
                 self.data.file.display(),
                 sample_rate,
            );
        }

        let mut samples = Vec::new();
        // Read interleaved audio.
        while let Ok(Some(frame)) = reader.read_dec_packet_itl() {
            samples.append(&mut frame.into_iter().map(|i| i as f32 / i16::MAX as f32).collect());
        }
        samples
    }
}
