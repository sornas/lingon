use super::LoadedFile;

use std::path::PathBuf;

pub struct Audio {
    samples: Vec<f32>,
    data: LoadedFile,
}

impl Audio {
    pub fn new(file: PathBuf) -> Self {
        let (data, bytes) = LoadedFile::new(file);
        let mut ret = Self {
            samples: Vec::new(),
            data,
        };
        ret.load_data(bytes);
        ret
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
        todo!()
    }
}
