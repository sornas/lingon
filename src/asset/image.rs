use super::{LoadedFile, Pixels};

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Image {
    pub width: Pixels,
    pub height: Pixels,
    pub texture_data: Vec<u8>,
    pub data: LoadedFile,
}

impl Image {
    pub fn new(file: PathBuf) -> Self {
        let (data, bytes) = LoadedFile::new(file);
        let mut ret = Self {
            width: 0,
            height: 0,
            texture_data: Vec::new(),
            data,
        };
        ret.load_data(bytes);
        ret
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
        let mut w: i32 = 0;
        let mut h: i32 = 0;
        let mut comp: i32 = 4;
        // SAFETY: stb_load_from_memory either succeeds or returns a null pointer
        unsafe {
            use stb_image::stb_image::bindgen::*;
            stbi_set_flip_vertically_on_load(1);
            let stb_image = stbi_load_from_memory(
                bytes.as_ptr(),
                bytes.len() as i32,
                &mut w,
                &mut h,
                &mut comp,
                4,
            );
            self.texture_data = Vec::from_raw_parts(
                stb_image as *mut u8,
                (w * h * 4) as usize,
                (w * h * 4) as usize,
            );
        }
        self.width = w as usize;
        self.height = h as usize;
    }
}
