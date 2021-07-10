use super::{LoadedFile};
use luminance_glyph::ab_glyph::FontArc;

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Font {
    pub font: FontArc,
    pub data: LoadedFile,
}

impl Font {
    pub fn new(file: PathBuf) -> Self {
        let (data, bytes) = LoadedFile::new(file);
        Self {
            font: FontArc::try_from_vec(bytes).unwrap(),
            data,
        }
    }

    pub fn reload(&mut self) -> bool {
        if let Some(bytes) = self.data.reload() {
            self.load_data(bytes);
            true
        } else {
            false
        }
    }

    fn load_data(&mut self, bytes: Vec<u8>) {
        self.font = FontArc::try_from_vec(bytes).unwrap();
    }
}
