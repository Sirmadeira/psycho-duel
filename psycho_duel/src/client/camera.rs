use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

pub struct ClientCameraPlugin;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        // Spawns our camera
        app.add_systems(Startup, spawn_camera);
    }
}



fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(MarkerPrimaryCamera)
        .insert(Name::new("MainCamera"));
}
