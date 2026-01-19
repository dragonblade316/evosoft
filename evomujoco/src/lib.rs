use std::{
    cell::RefCell, collections::HashMap, fs, hash::Hash, ops::Deref, path::PathBuf, rc::Rc,
    time::Duration,
};

use mujoco_rs::{mujoco_c::mjtJoint, prelude::MjData, viewer::MjViewer};

pub use mujoco_rs::prelude::MjModel;

pub struct Mujoco<'a> {
    model: &'a MjModel,
    data: Rc<RefCell<MjData<&'a MjModel>>>,
    viewer: MjViewer<&'a MjModel>,
    timestep: f64,
}

impl<'a> Mujoco<'a> {
    pub fn new(model: &'a MjModel) -> Self {
        let mode = Rc::new(model);

        // let model = MjModel::from_xml_string(&"").unwrap();
        //

        let data = Rc::new(RefCell::new(model.make_data()));
        let timestep = model.opt().timestep;
        let viewer = MjViewer::launch_passive(model, 0).expect("thing");

        Self {
            model,
            data,
            viewer,
            timestep,
        }
    }

    pub fn get_joints(&'a self) -> HashMap<String, Joint<'a>> {
        let ids = self.get_joint_ids();
        let mut map = HashMap::new();

        for i in ids {
            let joint = self.get_joint(i as usize);
            let name = self.get_joint_name(i as usize);
            map.insert(name, joint);
        }

        map
    }

    pub fn get_joint_ids(&self) -> Vec<i32> {
        (0..self.model.njnt() - 1).collect()
    }

    pub fn get_joint_name(&self, index: usize) -> String {
        self.model
            .id_to_name(mujoco_rs::mujoco_c::mjtObj::mjOBJ_JOINT, index as i32)
            .expect("how")
            .to_string()
    }

    pub fn get_index(&self, name: String) {}

    pub fn get_joint(&'a self, index: usize) -> Joint<'a> {
        match self.model.jnt_type().to_vec().get(index).unwrap() {
            mjtJoint::mjJNT_HINGE => Joint::Hinge {
                index,
                control: self,
            },
            mjtJoint::mjJNT_SLIDE => Joint::Slide {
                index,
                control: self,
            },
            mjtJoint::mjJNT_BALL => Joint::Ball {
                index,
                control: self,
            },
            mjtJoint::mjJNT_FREE => Joint::Free {
                index,
                control: self,
            },
        }
    }

    pub fn get_actuator(&'a self, name: String) -> Actuator<'a> {
        Actuator {
            name: name.clone(),
            index: self
                .model
                .name_to_id(mujoco_rs::mujoco_c::mjtObj::mjOBJ_ACTUATOR, &name),
            control: self.data.clone(),
        }
    }

    pub fn make_model(path: PathBuf) -> MjModel {
        let xml = fs::read_to_string(path).expect("invalid path");
        MjModel::from_xml_string(&xml).expect("invalid xml")
    }

    pub fn update(&mut self) {
        let mut data = self.data.borrow_mut();
        self.viewer.sync_data(&mut data);
        data.step();
        std::thread::sleep(Duration::from_secs_f64(self.timestep));
        self.viewer.render();
    }
}

enum Joint<'a> {
    Hinge {
        index: usize,
        control: &'a Mujoco<'a>,
    },
    Slide {
        index: usize,
        control: &'a Mujoco<'a>,
    },
    Ball {
        index: usize,
        control: &'a Mujoco<'a>,
    },
    Free {
        index: usize,
        control: &'a Mujoco<'a>,
    },
}

pub enum JointOutput {
    Scalar(f64),
    Ball {
        rx: f64,
        ry: f64,
        rz: f64,
    },
    Free {
        x: f64,
        y: f64,
        z: f64,
        rx: f64,
        ry: f64,
        rz: f64,
    },
}

impl Joint<'_> {
    pub fn get_qpos(&self) -> JointOutput {
        match self {
            Self::Slide { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let pos = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qpos;

                JointOutput::Scalar(pos.to_vec().get(0).unwrap().clone())
            }
            Self::Hinge { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let pos = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qpos;

                JointOutput::Scalar(pos.to_vec().get(0).unwrap().clone())
            }
            Self::Ball { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let pos = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qpos;

                let pose = pos.to_vec();

                JointOutput::Ball {
                    rx: pose.get(0).unwrap().clone(),
                    ry: pose.get(0).unwrap().clone(),
                    rz: pose.get(0).unwrap().clone(),
                }
            }
            Self::Free { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let pos = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qpos;

                let pose = pos.to_vec();

                JointOutput::Free {
                    x: pose.get(0).unwrap().clone(),
                    y: pose.get(1).unwrap().clone(),
                    z: pose.get(2).unwrap().clone(),
                    rx: pose.get(3).unwrap().clone(),
                    ry: pose.get(4).unwrap().clone(),
                    rz: pose.get(5).unwrap().clone(),
                }
            }
        }
    }

    pub fn get_qvel(&self) -> JointOutput {
        match self {
            Self::Slide { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let vel = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qvel;

                JointOutput::Scalar(vel.to_vec().get(0).unwrap().clone())
            }
            Self::Hinge { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let vel = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qvel;

                JointOutput::Scalar(vel.to_vec().get(0).unwrap().clone())
            }
            Self::Ball { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let vel = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qvel;

                let velocity = vel.to_vec();

                JointOutput::Ball {
                    rx: velocity.get(0).unwrap().clone(),
                    ry: velocity.get(1).unwrap().clone(),
                    rz: velocity.get(2).unwrap().clone(),
                }
            }
            Self::Free { index, control } => {
                let con = *control;
                let data = con.data.borrow();

                let vel = con
                    .data
                    .borrow()
                    .joint(&con.get_joint_name(index.clone()))
                    .expect("hopefully will work")
                    .view(&data)
                    .qvel;

                let velocity = vel.to_vec();

                JointOutput::Free {
                    x: velocity.get(0).unwrap().clone(),
                    y: velocity.get(1).unwrap().clone(),
                    z: velocity.get(2).unwrap().clone(),
                    rx: velocity.get(3).unwrap().clone(),
                    ry: velocity.get(4).unwrap().clone(),
                    rz: velocity.get(5).unwrap().clone(),
                }
            }
        }
    }
}

pub struct Actuator<'a> {
    name: String,
    index: i32,
    control: Rc<RefCell<MjData<&'a MjModel>>>,
}

impl Actuator<'_> {
    //no idea if this will work but here we are
    pub fn set_ctrl(&self, input: f64) {
        self.control.borrow_mut().ctrl_mut()[self.index as usize] = input;
    }
}
