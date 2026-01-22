use std::{
    cell::RefCell, collections::HashMap, fs, hash::Hash, ops::Deref, path::PathBuf, rc::Rc,
    time::Duration,
};

use image::{ImageBuffer, ImageReader, Luma, Rgb};
use mujoco_rs::{
    mujoco_c::mjtJoint,
    prelude::MjData,
    renderer::{self, MjRenderer},
    viewer::{MjViewer, egui::Image},
};

pub use mujoco_rs::prelude::MjModel;

pub struct Mujoco {
    model: Rc<MjModel>,
    data: Rc<RefCell<MjData<Rc<MjModel>>>>,
    viewer: MjViewer<Rc<MjModel>>,
    timestep: f64,
}

impl Mujoco {
    pub fn new(model: MjModel) -> Self {
        let model = Rc::new(model);

        let data = Rc::new(RefCell::new(MjData::new(model.clone())));
        let timestep = model.opt().timestep;
        let viewer = MjViewer::launch_passive(model.clone(), 0).expect("thing");

        Self {
            model: model.clone(),
            data,
            viewer,
            timestep,
        }
    }

    pub fn get_joints(&self) -> HashMap<String, Joint> {
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

    pub fn get_joint(&self, index: usize) -> Joint {
        match self.model.jnt_type().to_vec().get(index).unwrap() {
            mjtJoint::mjJNT_HINGE => Joint::Hinge {
                index,
                name: self.get_joint_name(index),
                control: self.data.clone(),
            },
            mjtJoint::mjJNT_SLIDE => Joint::Slide {
                index,
                name: self.get_joint_name(index),
                control: self.data.clone(),
            },
            mjtJoint::mjJNT_BALL => Joint::Ball {
                index,
                name: self.get_joint_name(index),
                control: self.data.clone(),
            },
            mjtJoint::mjJNT_FREE => Joint::Free {
                index,
                name: self.get_joint_name(index),
                control: self.data.clone(),
            },
        }
    }

    pub fn get_actuator(&self, name: String) -> Actuator {
        Actuator {
            name: name.clone(),
            index: self
                .model
                .name_to_id(mujoco_rs::mujoco_c::mjtObj::mjOBJ_ACTUATOR, &name),
            control: self.data.clone(),
        }
    }

    pub fn get_camera(&self, name: String, resolution: (u32, u32)) -> Camera {
        let (x, y) = resolution;
        Camera {
            x,
            y,
            data: self.data.clone(),
            renderer: MjRenderer::new(self.model.clone(), x as usize, y as usize, 256)
                .expect("Prob willwork"),
        }
    }

    pub fn update(&mut self) {
        let mut data = self.data.borrow_mut();
        self.viewer.sync_data(&mut data);
        data.step();
        std::thread::sleep(Duration::from_secs_f64(self.timestep));
        self.viewer.render();
    }
}

pub enum Joint {
    Hinge {
        index: usize,
        name: String,
        control: Rc<RefCell<MjData<Rc<MjModel>>>>,
    },
    Slide {
        index: usize,
        name: String,
        control: Rc<RefCell<MjData<Rc<MjModel>>>>,
    },
    Ball {
        index: usize,
        name: String,
        control: Rc<RefCell<MjData<Rc<MjModel>>>>,
    },
    Free {
        index: usize,
        name: String,
        control: Rc<RefCell<MjData<Rc<MjModel>>>>,
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

impl Joint {
    pub fn get_qpos(&self) -> JointOutput {
        match self {
            Self::Slide {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let pos = con
                    .borrow()
                    .joint(name)
                    .expect("hopefully will work")
                    .view(&data)
                    .qpos;

                JointOutput::Scalar(pos.to_vec().get(0).unwrap().clone())
            }
            Self::Hinge {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let pos = con
                    .borrow()
                    .joint(name)
                    .expect("hopefully will work")
                    .view(&data)
                    .qpos;

                JointOutput::Scalar(pos.to_vec().get(0).unwrap().clone())
            }
            Self::Ball {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let pos = con
                    .borrow()
                    .joint(name)
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
            Self::Free {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let pos = con
                    .borrow()
                    .joint(name)
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
            Self::Slide {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let vel = con
                    .borrow()
                    .joint(name)
                    .expect("hopefully will work")
                    .view(&data)
                    .qvel;

                JointOutput::Scalar(vel.to_vec().get(0).unwrap().clone())
            }
            Self::Hinge {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let vel = con
                    .borrow()
                    .joint(name)
                    .expect("hopefully will work")
                    .view(&data)
                    .qvel;

                JointOutput::Scalar(vel.to_vec().get(0).unwrap().clone())
            }
            Self::Ball {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let vel = con
                    .borrow()
                    .joint(name)
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
            Self::Free {
                index,
                name,
                control,
            } => {
                let con = control.clone();
                let data = con.borrow();

                let vel = con
                    .borrow()
                    .joint(name)
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

pub struct Actuator {
    name: String,
    index: i32,
    control: Rc<RefCell<MjData<Rc<MjModel>>>>,
}

impl Actuator {
    //no idea if this will work but here we are
    pub fn set_ctrl(&self, input: f64) {
        self.control.borrow_mut().ctrl_mut()[self.index as usize] = input;
    }
}

pub struct Camera {
    x: u32,
    y: u32,
    data: Rc<RefCell<MjData<Rc<MjModel>>>>,
    renderer: MjRenderer<Rc<MjModel>>,
}

impl Camera {
    pub fn render_rgb(&mut self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        self.renderer.sync(&mut self.data.borrow_mut());
        let raw = self.renderer.rgb_flat().expect("no camera detected");
        ImageBuffer::from_raw(self.x, self.y, raw.to_vec())
            .expect("This should not be possible since mujoco should always export something valid")
    }

    pub fn render_depth(&mut self) -> ImageBuffer<Luma<f32>, Vec<f32>> {
        self.renderer.sync(&mut self.data.borrow_mut());
        let raw = self.renderer.depth_flat().expect("no camera detected");
        ImageBuffer::from_raw(self.x, self.y, raw.to_vec())
            .expect("This should not be possible since mujoco should always export something valid")
    }
}
