use bevy::{DefaultPlugins, app::App};
use bevy_stl::StlPlugin;
use bevy_urdf::URDFPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((URDFPlugin::default(), StlPlugin));
}
