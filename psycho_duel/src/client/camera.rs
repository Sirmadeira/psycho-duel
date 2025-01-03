use super::protocol::PlayerMarker;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use lightyear::prelude::client::Predicted;
use lightyear::shared::replication::components::Controlled;

/// Centralization plugin, everything correlated to cameras will be inserted here
pub struct ClientCameraPlugin;

/// Marks our primary camera entity - Also know as the UI and Ingame camera
#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

/// Additional features outside of pan orbit
#[derive(Component, Reflect)]
pub struct CamFeatures {
    /// Button responsible for making our camera follow player
    follow_player_button: KeyCode,
    /// Should we follow or not
    follow_player_condition: bool,
}
impl Default for CamFeatures {
    fn default() -> Self {
        Self {
            follow_player_button: KeyCode::KeyR,
            follow_player_condition: false,
        }
    }
}

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        // Spawns our camera
        app.add_plugins(PanOrbitCameraPlugin);
        // Should occur right in the early stages
        app.add_systems(Startup, spawn_camera);
        // In pre update for responsiveness when toggled in the same frame i want to adjust
        app.add_systems(PreUpdate, toggle_cam_follow);
        // In update
        app.add_systems(Update, cam_follow_player.run_if(rc_follow_player));

        // Debug register
        app.register_type::<CamFeatures>();
        app.register_type::<PanOrbitCamera>();
    }
}

/// Run condition - Made it so cam follow player only runs the following system if cam feature follow player condition = true
fn rc_follow_player(cam_q: Query<&CamFeatures, With<MarkerPrimaryCamera>>) -> bool {
    if let Ok(cam_feat) = cam_q.get_single() {
        cam_feat.follow_player_condition
    } else {
        false
    }
}

/// Spawns our primary camera entity
fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3d::default())
        .insert(MarkerPrimaryCamera)
        .insert(Name::new("MainCamera"))
        .insert(
            Transform::from_translation(Vec3::new(0.0, 1.5, -5.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        )
        .insert(PanOrbitCamera::default())
        .insert(CamFeatures::default());
}

/// Turn on and the off the ability to follow player also prepare pan orbit settings
fn toggle_cam_follow(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cam_q: Query<(&mut PanOrbitCamera, &mut CamFeatures), With<MarkerPrimaryCamera>>,
) {
    if let Ok((mut pan_feat, mut cam_feat)) = cam_q.get_single_mut() {
        if keyboard_input.just_pressed(cam_feat.follow_player_button) {
            let new_condition = !cam_feat.follow_player_condition;
            cam_feat.follow_player_condition = new_condition;
            // If true prepare pan orbit camera to follow player
            if cam_feat.follow_player_condition {
                // Panning the camera changes the focus, and so you most likely want to disable
                // panning when setting the focus manually
                pan_feat.pan_sensitivity = 0.0;
                // If you want to fully control the camera's focus, set smoothness to 0 so it
                // immediately snaps to that location. If you want the 'follow' to be smoothed,
                // leave this at default or set it to something between 0 and 1.
                pan_feat.pan_smoothness = 0.0;
            } else {
                // Default values for panning camera
                pan_feat.pan_sensitivity = 1.0;
                pan_feat.pan_smoothness = 0.02;
            }
        }
    }
}

/// As the name implies it makes it so our panorbit camera follows our primary player
fn cam_follow_player(
    mut pan_q: Query<&mut PanOrbitCamera, With<MarkerPrimaryCamera>>,
    transform_q: Query<&Transform, (With<PlayerMarker>, With<Predicted>, With<Controlled>)>,
) {
    if let Ok(mut pan_cam) = pan_q.get_single_mut() {
        if let Ok(target) = transform_q.get_single() {
            // This makes it so the camera lerps into the given translation
            pan_cam.target_focus = target.translation;

            // Pan orbit camera requirement to force update
            pan_cam.force_update = true;
        }
    }
}
