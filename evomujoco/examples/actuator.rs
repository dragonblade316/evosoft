use std::time::SystemTime;

//Disclaimer: I had gemini write the mujoco xml. I dont know how to write this and frankly would
//rather just let onshape-to-robot do the work.
const EXAMPLE: &str = "<mujoco model=\"simple_rrbot\">
    <compiler angle=\"degree\" coordinate=\"local\"/>

    <worldbody>
        <light directional=\"true\" diffuse=\".8 .8 .8\" specular=\".2 .2 .2\" pos=\"0 0 5\" dir=\"0 0 -1\"/>
        
        <body name=\"base\" pos=\"0 0 0\">
            
            <body name=\"link1\" pos=\"0 0.1 0.4\">
                <geom name=\"link1_geom\" type=\"capsule\" size=\"0.04\" fromto=\"0 0 0 0 0 0.4\" rgba=\"1 0 0 1\"/>
                
                <body name=\"link2\" pos=\"0 0.1 0.4\">
                    <joint name=\"joint\" type=\"hinge\" axis=\"0 1 0\" pos=\"0 0 0\"/>
                    <geom name=\"link2_geom\" type=\"capsule\" size=\"0.04\" fromto=\"0 0 0 0 0 0.4\" rgba=\"0 1 0 1\"/>
                    
                    <site name=\"ee_site\" pos=\"0 0 0.4\" size=\"0.01\" rgba=\"1 1 1 1\"/>
                </body>
            </body>
        </body>
    </worldbody>

    <actuator>
        <position name=\"j_ctrl\" joint=\"joint\" kp=\"45\"/>
    </actuator>
</mujoco>";

fn main() {
    let model = evomujoco::MjModel::from_xml_string(EXAMPLE).unwrap();
    let mut mujoco = evomujoco::Mujoco::new(model);

    let act = mujoco.get_actuator("j_ctrl".to_string());
    let joint = mujoco.get_joint_from_name("joint".to_string());

    let start = SystemTime::now();

    loop {
        act.set_ctrl(f64::sin(
            SystemTime::now()
                .duration_since(start)
                .unwrap()
                .as_secs_f64(),
        ));
        println!("{:?}", joint.get_qpos());
        mujoco.update();
    }
}
