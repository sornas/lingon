use super::LoadedFile;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

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
}

impl Audio {
    pub fn new(file: PathBuf) -> Self {
        let (data, bytes) = LoadedFile::new(file);
        Self {
            samples: Arc::new(RwLock::new(load_data(bytes))),
            data,
        }
    }

    pub fn samples(&self) -> Arc<RwLock<Samples>> {
        Arc::clone(&self.samples)
    }

    pub fn reload(&mut self) -> bool {
        if let Some(bytes) = self.data.reload() {
            *self.samples.write().unwrap() = load_data(bytes);
            true
        } else {
            false
        }
    }
}

pub fn load_data(bytes: Vec<u8>) -> Samples {
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
