# EvoMuJoCo
A very simple wrapper around [mujoco-rs](https://crates.io/crates/mujoco-rs) meant to make the api easier to use and fit a little better with rust conventions.

If you want a very simple and quick way to set up and interact with a mujoco simulation. Then this crate is for you. If you need access to all of mujoco's features or the absolute max in performance, I would recomend that you look at [mujoco-rs](https://crates.io/crates/mujoco-rs).

This is a library I'm developing in my free time for my own use. New features are coming but they are limited by whatever freetime I have between my studies. If you want something implemented feel free to make an issue or pull request on the github.

Keep in mind that this library is still a work in progress so there will likely be breaking changes with each version.

## Install
In order for the package to run properly, MuJoCo must be installed and the following env var must be set: 

```bash
export MUJOCO_DYNAMIC_LINK_DIR=/path/mujoco/lib/
```

## Currently Supported:
- Getting qpos and qvel from joints.
- Controllering basic actuators.
- Easy rendering of mujoco cameras.
- Viewer powered by mujoco-rs.

## Planed
- Supporting other sensors (I currently plan on implementing the IMU but if you need another sensor implemented, make an issue on the github).
- Getting xpos and xvel from joints (I just have not needed this yet)
- Maybe switching to nalgebra types to repersent transforms.
- Custom file loader to fix the shortcomings of mj_loadXML.

## Example
```rust
//I borrowed this mujoco xml from the mujoco-rs readme. More advanced examples on the way.
const EXAMPLE: &str = "
<mujoco>
  <worldbody>
    <light ambient=\"0.2 0.2 0.2\"/>
    <body name=\"ball\">
        <geom name=\"green_sphere\" size=\".1\" rgba=\"0 1 0 1\" solref=\"0.004 1.0\"/>
        <joint name=\"ball_joint\" type=\"free\"/>
    </body>

    <geom name=\"floor1\" type=\"plane\" size=\"10 10 1\" euler=\"15 4 0\" solref=\"0.004 1.0\"/>
    <geom name=\"floor2\" type=\"plane\" pos=\"15 -20 0\" size=\"10 10 1\" euler=\"-15 -4 0\" solref=\"0.004 1.0\"/>

  </worldbody>
</mujoco>
";

fn main() {
    let model = evomujoco::MjModel::from_xml_string(EXAMPLE).unwrap();
    let mut mujoco = evomujoco::Mujoco::new(model);

    loop {
        mujoco.update();
    }
}
```
