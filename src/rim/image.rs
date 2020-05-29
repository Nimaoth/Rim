// #[macro_use]
// // mod crate::opengl_macros;
// use crate::opengl_macros::*;

use std::path::*;
use std::rc::Rc;

pub struct Image {
    pub path: std::path::PathBuf,
    pub renderer_id: usize,
    pub width: usize,
    pub height: usize,
}

impl Image {
    pub fn new(path: &Path) -> Result<Rc<Image>, ()> {
        let image = match image::open(path) {
            Ok(img) => img,
            Err(_) => return Err(()),
        };

        let img_data = image.to_rgb();
        let (width, height, format, data_format, data_type) = (
            img_data.width(),
            img_data.height(),
            gl::RGB8,
            gl::RGB,
            gl::UNSIGNED_BYTE,
        );

        let mut tex_id: u32 = 0;
        GL!(GenTextures(1, &mut tex_id));
        GL!(BindTexture(TEXTURE_2D, tex_id));
        GL!(TexParameteri(
            TEXTURE_2D,
            TEXTURE_MIN_FILTER,
            NEAREST as i32
        ));
        GL!(TexParameteri(
            TEXTURE_2D,
            TEXTURE_MAG_FILTER,
            NEAREST as i32
        ));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32));
        GL!(TexImage2D(
            TEXTURE_2D,
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            data_format,
            data_type,
            img_data.as_ptr() as *const std::ffi::c_void
        ));
        GL!(BindTexture(TEXTURE_2D, 0));

        let image = Rc::new(Image {
            path: path.to_owned(),
            renderer_id: tex_id as usize,
            width: width as usize,
            height: height as usize,
        });

        return Ok(image);
    }

    pub fn reload_from_disk(&self) -> Result<(), ()> {
        println!("reloading image {:?}", self.path);
        let image = match image::open(&self.path) {
            Ok(img) => img,
            Err(_) => return Err(()),
        };

        let (img_data, width, height, format, data_format, data_type) = match image.as_rgb8() {
            Some(rgb) => (
                rgb.as_ref(),
                rgb.width(),
                rgb.height(),
                gl::RGB8,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            None => return Err(()),
        };

        GL!(BindTexture(TEXTURE_2D, self.renderer_id as u32));
        GL!(TexImage2D(
            TEXTURE_2D,
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            data_format,
            data_type,
            img_data.as_ptr() as *const std::ffi::c_void
        ));

        Ok(())
    }
}
