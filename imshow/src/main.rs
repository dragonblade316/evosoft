use evobridge::Session;
use evotypes_rs::evosoft::image::Image;
use show_image::create_window;
use tokio::runtime::Runtime;

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {

    let rt  = Runtime::new().expect("Failed to create Tokio runtime");

    // Create a window with default options and display the image.
    let window = create_window("image", Default::default())?;
    // window.set_image("image-001", image)?;

    // 2. Block on the main async function
    rt.block_on(async move {
        println!("Runtime created and running the main async block!");

        let session = Session::new(None, None).await;
        let sub = session.subscribe("evocam/cam0").build().await;

        loop {
            let frame = sub.recv_async().await.unwrap();
            let thing: Image = frame.payload().unwrap();
            let img = thing.get_image();

            window.set_image("image", img).unwrap();
            
        }
    });

    Ok(())
}
