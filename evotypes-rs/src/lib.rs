use std::ops::Deref;

use image::{ImageBuffer, Pixel, Rgb};
use derive_name::Name;

macro_rules! buf {
    ($module:literal) => {
        use crate::Name;
        include!(concat!(env!("OUT_DIR"), $module));
    };
}

pub mod evosoft {
    // pub use derive_name::Name;

    pub mod geometry {
        buf!("/evosoft.geometry.rs");
    }

    pub mod image {
        buf!("/evosoft.image.rs");
    }

    pub mod motors {
        buf!("/evosoft.motors.rs");
    }
}
impl evosoft::image::Image {
    pub fn from_image(img: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        Self {
            width: img.width(),
            height: img.height(),
            encoding: evosoft::image::Encoding::Rgb8.into(),
            data: img.into_raw(),
        }
    }

    pub fn get_image(self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        ImageBuffer::from_raw(self.width, self.height, self.data).expect("If you are too big for vec<u8>, you need help and should not be sending this")
    }
}
