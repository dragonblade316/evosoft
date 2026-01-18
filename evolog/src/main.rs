use evobridge::Session;
use anyhow::Result;
use evotypes_rs::evosoft::{geometry::Twist, image::Image};
use rerun::{Angle, Rotation3D, components::RotationAxisAngle, Vec2D, Vec3D, components::Translation3D, external::image::Frame};

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
            "protobuf/Pose2d" => {
                let p2d: evotypes_rs::evosoft::geometry::Pose2d = data.payload().expect("I really should add error handling");
                let translation = Translation3D::new(p2d.x as f32, p2d.y as f32, 0.0);
                let rot = Rotation3D::AxisAngle(RotationAxisAngle::new(Vec3D::new(0.0, 0.0, 1.0), Angle::from_radians(p2d.theta as f32)));
                
                let _ = rec.log(data.key_expr(), &rerun::Transform3D::from_translation(translation).with_rotation(rot));
            }

            "protobuf/Image" => {
                let imgdata: Image = data.payload().expect("hopefully no issues");
                let img = imgdata.data;

                let _ = rec.log(
                    data.key_expr(), 
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
