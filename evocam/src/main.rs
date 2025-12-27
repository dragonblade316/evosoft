use std::collections::HashMap;

use image::codecs::png::PngEncoder;
use nokhwa::{Camera, pixel_format::RgbFormat, utils::{CameraIndex, RequestedFormat, RequestedFormatType}};

#[tokio::main]
async fn main() {
    // first camera in system
    let index = CameraIndex::Index(0); 

    let session = evobridge::Session::new(None, None).await;
    let publisher = session.get_publisher("evocam/cam0").await;

    // request the absolute highest resolution CameraFormat that can be decoded to RGB.
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    // make the camera
    let mut camera = Camera::new(index, requested).unwrap();
    camera.open_stream().unwrap();

    loop {
        let frame = camera.frame().unwrap();   
        let decoded: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = frame.decode_image::<RgbFormat>().unwrap();
        let _ = publisher.put(evotypes_rs::evosoft::image::Image::from_image(decoded)).await;
    }
    // get a frame
    let frame = camera.frame().unwrap();
    // println!("Captured Single Frame of {}", frame.buffer().len());
    // // decode into an ImageBuffer
    // let decoded: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = frame.decode_image::<RgbFormat>().unwrap();
    // println!("Decoded Frame of {}", decoded.len());
}
