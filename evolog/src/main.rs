use evobridge::Session;
use anyhow::Result;
use evotypes_rs::evosoft::{geometry::Twist, image::Image};
use rerun::external::image::Frame;

#[tokio::main]
async fn main() -> Result<()>{
    let session = Session::new(None, None).await;
    let rec = rerun::RecordingStreamBuilder::new("rerun_example_minimal").connect_grpc()?;

    let sub = session.subscribe("**").build().await;

    use embedded_fps::{FPS, StdClock};
    let std_clock = StdClock::default();
    let mut fps_counter = embedded_fps::FPS::<60, _>::new(std_clock);
    
    loop {
        let data = sub.recv_async().await.unwrap();

        match data.encoding().to_string().as_str() {
            "protobuf/Image" => {
                let imgdata: Image = data.payload().expect("hopefully no issues");
                let img = imgdata.data;

                let _ = rec.log(
                    "camera/image", 
                    &rerun::Image::from_color_model_and_bytes(
                        img, 
                        [imgdata.width, imgdata.height], 
                        rerun::ColorModel::RGB, 
                        rerun::ChannelDatatype::U8
                    ))?;
            },
            _ => {}
        }

        println!("{}", fps_counter.tick());
    }
}
