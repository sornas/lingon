use super::LoadedFile;

use std::{ops::Deref, path::PathBuf};
use std::sync::{Arc, RwLock};

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
}

impl Audio {
    pub fn new(file: PathBuf) -> Self {
        let (data, bytes) = LoadedFile::new(file);
        let mut ret = Self {
            samples: Samples::new(),
            data,
        };
        ret.load_data(bytes);
        ret
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
        let (header, data) = wav::read(&mut std::io::Cursor::new(bytes)).unwrap();
        println!("Read wav: {:?}", header);
        let mut samples = self.samples.write().unwrap();
        match data {
            wav::BitDepth::ThirtyTwoFloat(data) => *samples = data,
            _ => todo!(),
        }
    }
}
