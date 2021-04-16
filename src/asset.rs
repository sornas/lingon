use std::path::Path;

/// A marker type for the unit pixels.
pub type Pixels = usize;

#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Image {
    /// A fast and easy way to convert bytes to an image.
    pub fn load_from_memory(bytes: &[u8]) -> Option<Self> {
        // SAFETY: stbi either succeeds or returns a null pointer
        let mut w: i32 = 0;
        let mut h: i32 = 0;
        let mut comp: i32 = 4;
        unsafe {
            use stb_image::stb_image::bindgen::*;
            stbi_set_flip_vertically_on_load(1);
            let data = stbi_load_from_memory(
                bytes.as_ptr(),
                bytes.len() as i32,
                &mut w,
                &mut h,
                &mut comp,
                4,
            );
            if data.is_null() {
                None
            } else {
                let data =
                    Vec::from_raw_parts(data as *mut u8, (w * h * 4) as usize, (w * h * 4) as usize);
                Some(Image {
                    width: w as usize,
                    height: h as usize,
                    data,
                })
            }
        }
    }

    pub fn load_from_file(file: &Path) -> Option<Self> {
        let bytes = std::fs::read(file).unwrap();
        Self::load_from_memory(&bytes)
    }
}
