use super::LoadedFile;

use std::convert::{TryFrom, TryInto};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// The different kinds of files we can open.
#[derive(Copy, Clone)]
pub enum AudioFileKind {
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
pub struct Samples {
    data: Vec<f32>,
    sample_rate: u32,
}

impl Samples {
    pub fn new(data: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            data,
            sample_rate,
        }
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

pub struct Audio {
    samples: Arc<RwLock<Samples>>,
    data: LoadedFile,
    kind: AudioFileKind,
}

impl Audio {
    pub fn new(file: PathBuf) -> Option<Self> {
        let kind = file.extension()?.to_str()?.try_into().ok()?;
        let (data, bytes) = LoadedFile::new(file);
        Some(Self {
            samples: Arc::new(RwLock::new(load_data(bytes, kind))),
            data,
            kind,
        })
    }

    pub fn samples(&self) -> Arc<RwLock<Samples>> {
        Arc::clone(&self.samples)
    }

    pub fn reload(&mut self) -> bool {
        if let Some(bytes) = self.data.reload() {
            *self.samples.write().unwrap() = load_data(bytes, self.kind);
            true
        } else {
            false
        }
    }
}

pub fn load_data(bytes: Vec<u8>, kind: AudioFileKind) -> Samples {
    match kind {
        AudioFileKind::Ogg => load_ogg(bytes),
        AudioFileKind::Wav => load_wav(bytes),
    }
}

pub fn load_wav(bytes: Vec<u8>) -> Samples {
    let (header, data) = wav::read(&mut std::io::Cursor::new(bytes)).unwrap();
    let data = match data {
        wav::BitDepth::ThirtyTwoFloat(data) =>  data,
        _ => todo!("Only WAV containing floats are currently supported"),
    };
    Samples {
        data,
        sample_rate: header.sampling_rate,
    }
}

pub fn load_ogg(bytes: Vec<u8>) -> Samples {
    let mut reader = lewton::inside_ogg::OggStreamReader::new(Cursor::new(&bytes)).unwrap();

    let mut data = Vec::new();
    // Read interleaved audio.
    while let Ok(Some(frame)) = reader.read_dec_packet_itl() {
        data.append(&mut frame.into_iter().map(|i| i as f32 / i16::MAX as f32).collect());
    }
    Samples {
        data,
        sample_rate: reader.ident_hdr.audio_sample_rate,
    }
}
