use bevy::prelude::*;
use bevy::log::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(LogPlugin{
        // Utilized to set bevy log level 
        level: bevy::log::Level::TRACE,..default()}));

    app.add_systems(Startup, spawn_camera);

}


fn spawn_camera(mut commands: Commands){
    
    commands.spawn(Camera3dBundle::default());

}