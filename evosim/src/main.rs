//! Example of using views.
//! The example shows how to obtain a [`MjJointInfo`] struct that can be used
//! to create a (temporary) [`MjJointView`] to corresponding fields in [`MjData`].
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;


use evobridge::{Message, Publisher, Session, SubMessage, Subscriber};
use mujoco_rs::viewer::MjViewer;
use mujoco_rs::prelude::*;
use serde::Deserialize;

// mod motues_control;

const EXAMPLE_MODEL: &str = "
<mujoco>
  <worldbody>
    <light pos=\"0 0 1\"/>
    
    <body name=\"table\" pos=\"0 0 0.025\">
      <geom name=\"plate\" type=\"box\" size=\"0.25 0.2 0.025\" rgba=\".8 .8 .8 1\"/>
      <geom name=\"bound0\" type=\"box\" size=\".25 .01 .05\" pos=\"0 -.19 .075\"/>
      <geom name=\"bound1\" type=\"box\" size=\".01 .19 .05\" pos=\"-.24 .01 .075\"/>
    </body>

    <body name=\"booklink\" pos=\"0 0 0.065\">
      <freejoint/>
      <geom name=\"book\" type=\"box\" size=\".1 .05 .0125\" rgba=\"0 1 0 1\" mass=\".1\"/>
    </body>  

    <body name=\"spatulalink\" pos=\"0 0 .2\">
        <joint name=\"transX\" type=\"slide\" axis=\"1 0 0\" limited=\"true\" range=\"-.6 .6\"/>
        <joint name=\"transY\" type=\"slide\" axis=\"0 1 0\" limited=\"true\" range=\"-.6 .6\"/>
        <joint name=\"transZ\" type=\"slide\" axis=\"0 0 1\" limited=\"true\" range=\"-.6 .6\"/>
        <joint name=\"hingeX\" type=\"hinge\" axis=\"1 0 0\" limited=\"true\" range=\"-3.2 3.2\"/>
        <joint name=\"hingeY\" type=\"hinge\" axis=\"0 1 0\" limited=\"true\" range=\"-3.2 3.2\"/>
        <joint name=\"hingeZ\" type=\"hinge\" axis=\"0 0 1\" limited=\"true\" range=\"-3.2 3.2\"/>
        <geom name=\"spatula\" type=\"box\" size=\".05 .05 5e-4\" rgba=\"1 1 0 1\" friction=\".1 .1 .1\" mass=\".1\"/>
    </body>
  </worldbody>
  <actuator>
    <position ctrllimited=\"true\" ctrlrange=\"-.6 .6\" joint=\"transX\"/>
    <position ctrllimited=\"true\" ctrlrange=\"-.6 .6\" joint=\"transY\"/>
    <position ctrllimited=\"true\" ctrlrange=\"-.6 .6\" joint=\"transZ\"/>
    <position ctrllimited=\"true\" ctrlrange=\"-3.2 3.2\" joint=\"hingeX\"/>
    <position ctrllimited=\"true\" ctrlrange=\"-3.2 3.2\" joint=\"hingeY\"/>
    <position ctrllimited=\"true\" ctrlrange=\"-3.2 3.2\" joint=\"hingeZ\"/>
  </actuator>
</mujoco>
";

#[derive(Deserialize)]
enum ActuatorType {
  Motor,
  Position,
  Velocity,
}

#[derive(Deserialize)]
struct ActuatorConfigEntry {
  actuator: ActuatorType,
  joint: String
}

#[derive(Deserialize)]
struct EvosimConfig {
  path: PathBuf,
  actuators: HashMap<String, ActuatorConfigEntry>
} 

// #[derive(Debug)]
struct ActuatorHandler<'a, T: Message + Default> {
  // session: Session,
  rx: tokio::sync::watch::Receiver<T>,
  pub_pos: Publisher<'a>,
  pub_vel: Publisher<'a>
}

impl<'a, T: Message + Default + 'static + Clone + Debug> ActuatorHandler<'a, T> {
  async fn new(name: String, session: &'a Session) -> Self {
    let (tx, mut rx) = tokio::sync::watch::channel(T::default());
    
    let sub = session.subscribe(format!("{}/cmd", name).as_str()).build().await;

    let pub_pos = session.get_publisher("todo").await;
    let pub_vel = session.get_publisher("todo_v").await;

    
    tokio::spawn(async move {
      loop {
        // println!("waiting for data");
        let data = sub.recv_async().await.expect("who knows").payload().expect("could not decode payload");
        // println!("data: {:?}", data);
        let _ = tx.send(data);
      }
      
    });

    Self {rx, pub_pos, pub_vel}
  }

  fn update(&self, pos: Vec<f64>, vel: Vec<f64>) -> T {

    return (*self.rx.borrow()).clone();
  }
}

// #[derive(Debug)]
enum Actuator<'a> {
    //Moteus is depricated for now bc I dont want to finish it
    //Moteus{joint: String, motor: String}, 
    //
    //Position and velocity using cascading pid loops
    Motor{joint: String, name: String},
    //Position and velocity.
    Position{joint: String, pos: String, handler: ActuatorHandler<'a, evotypes_rs::evosoft::geometry::Angle>},
    Velocity{joint: String, vel: String, handler: ActuatorHandler<'a, evotypes_rs::evosoft::geometry::AngularVelocity>},
}

impl Actuator<'_> {
  fn update(&self, data: &MjData<&MjModel>) -> f64 {
    match self {
      Actuator::Motor{joint, name} => unimplemented!(),
      Actuator::Position {joint, pos, handler } => {
        let jd = data.joint(joint).expect("joint does not exist"); 
        let thing = handler.update(jd.view(&data).qpos.to_vec(), jd.view(&data).qvel.to_vec());
        thing.radians
      },
      Actuator::Velocity { joint, vel, handler } => {
        let jd = data.joint(joint).expect("joint does not exist"); 
        let thing = handler.update(jd.view(&data).qpos.to_vec(), jd.view(&data).qvel.to_vec());
        thing.radians_per_second
      },
    }
  } 

  fn act_name(&self, model: &MjModel) -> usize {
    match self {
      Actuator::Motor{joint, name} => unimplemented!(),
      Actuator::Position {joint, pos, handler } => model.actuator(pos).expect("act does not exist").id,
      Actuator::Velocity { joint, vel, handler } => model.actuator(vel).expect("act does not exist").id
    }
  }
}

#[tokio::main]
async fn main() {
 
  let path = "evosim.toml";
  let config_string = fs::read_to_string(path).unwrap();
  let config: EvosimConfig = toml::from_str(&config_string).unwrap();
 
  let session = Session::new(None, None).await;

  let mut actuators = Vec::new();

  for i in config.actuators.keys() {
    let entry = config.actuators.get(i).expect("not possible to reach this error");

    let value = match entry.actuator {
      ActuatorType::Position => Actuator::Position { joint: entry.joint.clone(), pos: i.clone(), handler: ActuatorHandler::new(i.clone(), &session).await },
      ActuatorType::Velocity => Actuator::Velocity { joint: entry.joint.clone(), vel: i.clone(), handler: ActuatorHandler::new(i.clone(), &session).await },
      
      _ => unimplemented!()
    };

    actuators.push(value);
  }

  let content = fs::read_to_string(config.path).unwrap();

  let model = MjModel::from_xml_string(&content).expect("could not load the model");
  let mut data = model.make_data();  // or MjData::new(&model);

  let mut viewer = MjViewer::launch_passive(&model, 0)
      .expect("could not launch the viewer");

  /* Obtain the timestep through the wrapped mjModel */
  let timestep = model.opt().timestep;

  while viewer.running() {
      //dont need mut yet
      for i in &actuators {
        let ctrl = i.update(&data);
        let id = i.act_name(&model);

        println!("ctrl {}", ctrl);
        data.ctrl_mut()[id] = ctrl;
      }

      viewer.sync_data(&mut data);
      data.step();

      std::thread::sleep(Duration::from_secs_f64(timestep));
      viewer.render();
  }
}
