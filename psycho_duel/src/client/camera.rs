use bevy::prelude::*;

/// Marks our main camera entity
#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

/// Centralization plugin, everything correlated to cameras will be inserted here
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
