use std::collections::HashMap;

use image::codecs::png::PngEncoder;
use nokhwa::{Camera, pixel_format::RgbFormat, utils::{CameraFormat, CameraIndex, RequestedFormat, RequestedFormatType, Resolution}};

#[tokio::main]
async fn main() {
    // first camera in system
    let index = CameraIndex::Index(0); 

    let session = evobridge::Session::new(None, None).await;
    let publisher = session.get_publisher("evocam/cam0").await;

    publisher.get_matching_listener().await.with_mut_callback(|m| {
        match m.matching() {
            true => println!("an entity is matching"),
            false => println!("there is no entity matching")
        };
    });

    // use embedded_fps::{FPS, StdClock};
    // let std_clock = StdClock::default();
    // let mut fps_counter = embedded_fps::FPS::<60, _>::new(std_clock);
    //
    
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    // make the camera
    let mut camera = Camera::new(index, requested).unwrap();
    camera.open_stream().unwrap();

    println!("{}", camera.camera_format());

    loop {
        let frame = camera.frame().unwrap();   
        let decoded: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = frame.decode_image::<RgbFormat>().unwrap();
        let img =  evotypes_rs::evosoft::image::Image::from_image(decoded);

        let _ = publisher.put(img).await.expect("there is no err case");

        // println!("{}", fps_counter.tick());
    }
}
