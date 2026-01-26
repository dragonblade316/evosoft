use evomujoco::Mujoco;

const EXAMPLE_MODEL: &str = "<mujoco>
  <worldbody>
    <light diffuse=\".5 .5 .5\" pos=\"0 0 3\" dir=\"0 0 -1\"/>
    <geom type=\"plane\" size=\"1 1 0.1\" rgba=\".9 0 0 1\"/>
    <body pos=\"0 0 1\">
      <joint name=\"free\" type=\"free\"/>
      <geom type=\"box\" size=\".1 .2 .3\" rgba=\"0 .9 0 1\"/>
    </body>
  </worldbody>
</mujoco>";

fn main() {
    let model = evomujoco::MjModel::from_xml_string(EXAMPLE_MODEL).unwrap();

    let mut mujoco = Mujoco::new(model);
    let joint = mujoco.get_joint_from_name("free".to_string());

    loop {
        mujoco.update();
        match joint.get_qpos() {
            evomujoco::JointOutput::Free(j) => println!("{:#?}", j),
            _ => {}
        }
    }
}
