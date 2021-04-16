use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Data {
    pub file: PathBuf,
    pub bytes: Vec<u8>,
    pub last_modified: SystemTime,
}

impl Data {
    //FIXME error handling
    pub fn load(file: PathBuf) -> Self {
        let bytes = std::fs::read(&file).unwrap();
        let last_modified = std::fs::metadata(&file).unwrap().modified().unwrap();
        Self {
            file,
            bytes,
            last_modified,
        }
    }

    //FIXME error handling
    pub fn reload(&mut self) {
        let last_modified = std::fs::metadata(&self.file).unwrap().modified().unwrap();
        if last_modified != self.last_modified {
            let bytes = std::fs::read(&self.file).unwrap();
            self.bytes = bytes;
            self.last_modified = last_modified;
        }
    }
}

/// A marker type for the unit pixels.
pub type Pixels = usize;

#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Data,
}

impl Image {
    //FIXME error handling
    pub fn load(file: PathBuf) -> Self {
        let mut w: i32 = 0;
        let mut h: i32 = 0;
        let mut comp: i32 = 4;
        let mut data = Data::load(file);

        // SAFETY: stb_load_from_memory either succeeds or returns a null pointer
        unsafe {
            use stb_image::stb_image::bindgen::*;
            stbi_set_flip_vertically_on_load(1);
            let stb_image = stbi_load_from_memory(
                data.bytes.as_ptr(),
                data.bytes.len() as i32,
                &mut w,
                &mut h,
                &mut comp,
                4,
            );
            data.bytes = Vec::from_raw_parts(stb_image as *mut u8, (w * h * 4) as usize, (w * h * 4) as usize);
            Image {
                width: w as usize,
                height: h as usize,
                data,
            }
        }
    }
}
