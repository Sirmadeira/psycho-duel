use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

/// Centralization plugin, everything correlated to cameras will be inserted here
pub struct ClientCameraPlugin;

/// Marks our primary camera entity - Also know as the UI and Ingame camera
#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        // Spawns our camera
        app.add_plugins(PanOrbitCameraPlugin);
        app.add_systems(Startup, spawn_camera);
    }
}

/// Spawns our primary camera entity
fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(MarkerPrimaryCamera)
        .insert(Name::new("MainCamera"))
        .insert(Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)))
        .insert(PanOrbitCamera::default());
}
