use image::DynamicImage;
use image_ascii::TextGenerator;

const EXAMPLE: &str = "<mujoco model=\"simple_camera_scene\">
    <visual>
        <global offwidth=\"1920\" offheight=\"1080\"/>
    </visual>

    <worldbody>
        <light diffuse=\".5 .5 .5\" pos=\"0 0 3\" dir=\"0 0 -1\"/>

        <geom type=\"plane\" size=\"5 5 0.1\" rgba=\".9 .9 .9 1\"/>

        <body pos=\"0 0 0.2\">
            <geom type=\"box\" size=\"0.2 0.2 0.2\" rgba=\"1 0 0 1\"/>
        </body>

        <camera name=\"main_camera\" pos=\"1.5 1.5 1.0\" euler=\"0 45 135\" resolution=\"1920 1080\"/>
    </worldbody>
</mujoco>";

fn main() {
    let model = evomujoco::MjModel::from_xml_string(EXAMPLE).expect("Could not load data");
    let mut mujoco = evomujoco::Mujoco::new(model);

    let mut cam = mujoco.get_camera("main_camera".to_string(), (1920, 1080));

    loop {
        mujoco.update();
        let img = cam.render_rgb();

        //render rgb returns a ImgBuffer<Rgb<u8>, Vec<u8>> by default so you may need to convert to
        //dynamic image
        let dynamic: DynamicImage = img.into();

        //This is the best way I could think of to display the image easily on most systems
        println!(
            "{}",
            artem::convert(dynamic, &artem::config::Config::default())
        );

        //keep in mind that since the camera is static the image will not change over time. Also
        //with this mjcf the camera is upside down for some reason.
    }
}
