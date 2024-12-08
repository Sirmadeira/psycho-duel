use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::Actionlike;

use super::egui::EguiWantsFocus;

/// Centralization plugin, everything correlated to cameras will be inserted here
pub struct ClientCameraPlugin;

/// Marks our primary camera entity - Also know as the UI and Ingame camera
#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

/// Essential struct that sets  a series of possible variable we can play with to make our camera more adjustable
#[derive(Reflect, Component, Debug)]
pub struct CamInfo {
    pub mouse_sens: f32,
    pub zoom_enabled: bool,
    pub zoom: Zoom,
    pub zoom_sens: f32,
    pub yaw_limit: Option<(f32, f32)>,
    pub pitch_limit: Option<(f32, f32)>,
}

/// Sets the zoom bounds (min & max)
#[derive(Reflect, Component, Debug)]
pub struct Zoom {
    pub min: f32,
    pub max: f32,
    pub radius: f32,
}

impl Zoom {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            radius: (min + max) / 2.0,
        }
    }
}

/// Actionlike - This guy represents the current actions we have available to us also know as states of camera
/// When utilizing actionlikes dont forget to add the inputmanagerplugin from leafwing
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(Axis)]
    Zoom,
    #[actionlike(DualAxis)]
    Pan,
}

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        // Addition of leafwing input manager plugin for camera movement
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        // Spawns our camera
        app.add_systems(Startup, spawn_camera);
        // In update for simplicity  Chain  - To ensure that we apply zoom in the same frame for smoothing
        app.add_systems(
            Update,
            (adjust_zoom_info, orbit_camera)
                .run_if(rc_egui_wants_focus)
                .chain(),
        );
        // Debug for leafwing
        app.register_type::<InputMap<CameraMovement>>();
    }
}

/// Run condition - Ensures that our camera doesnt zoom or move when we playing with egui
fn rc_egui_wants_focus(egui_wants_focus: Res<EguiWantsFocus>) -> bool {
    if egui_wants_focus.prev && egui_wants_focus.curr {
        //If egui wants focus in both prev and curr frame turn off the systems responsible for orbiting
        false
    } else {
        // Run if not in egui
        true
    }
}

/// Spawns our primary camera entity
fn spawn_camera(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_dual_axis(CameraMovement::Pan, MouseMove::default())
        .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y);

    commands
        .spawn(Camera3dBundle::default())
        .insert(MarkerPrimaryCamera)
        .insert(Name::new("MainCamera"))
        .insert(InputManagerBundle::with_map(input_map))
        .insert(CamInfo {
            mouse_sens: 0.75,
            zoom_enabled: true,
            zoom: Zoom::new(5.0, 10.0),
            zoom_sens: 0.1,
            yaw_limit: None,
            pitch_limit: Some((-std::f32::consts::PI / 2.0, std::f32::consts::PI / 20.0)),
        });
}

/// Responsible for changing zoom information
fn adjust_zoom_info(
    mut cam_q: Query<(&mut CamInfo, &ActionState<CameraMovement>), With<MarkerPrimaryCamera>>,
) {
    if let Ok((mut cam, camera_movement)) = cam_q.get_single_mut() {
        let scroll = camera_movement.value(&CameraMovement::Zoom);
        // If there is any type of scroll movement, we will capture it and adjust our zoom according to it
        // We use double minus here because of -y equals go back, but you know you wanna increase Z in this case
        if scroll.abs() > 0.0 {
            let new_radius = cam.zoom.radius - scroll * cam.zoom.radius * cam.zoom_sens;
            cam.zoom.radius = new_radius.clamp(cam.zoom.min, cam.zoom.max);
        }
    }
}

/// Enables camera to orbit according to mouse movement, is worth mentioning this camera will remain in place
/// She is undraggable
fn orbit_camera(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cam_q: Query<
        (&CamInfo, &ActionState<CameraMovement>, &mut Transform),
        With<MarkerPrimaryCamera>,
    >,
) {
    if let (Ok(window), Ok((cam_info, camera_movement, mut cam_transform))) =
        (window_q.get_single(), cam_q.get_single_mut())
    {
        // Accumulate mouse motion into a rotation vector
        let rotation_delta: Vec2 = camera_movement.axis_pair(&CameraMovement::Pan);
        if rotation_delta != Vec2::ZERO {
            // Calculate normalized rotation deltas
            let delta_x =
                (rotation_delta.x / window.width()) * std::f32::consts::PI * cam_info.mouse_sens;
            let delta_y =
                (rotation_delta.y / window.height()) * std::f32::consts::PI * cam_info.mouse_sens;

            // Retrieve current yaw and pitch
            let (yaw, pitch, _) = cam_transform.rotation.to_euler(EulerRot::YXZ);

            // Apply yaw limit if set
            let new_yaw = if let Some((min_yaw, max_yaw)) = cam_info.yaw_limit {
                (yaw - delta_x).clamp(min_yaw, max_yaw)
            } else {
                yaw - delta_x
            };

            // Apply pitch limit if set
            let new_pitch = if let Some((min_pitch, max_pitch)) = cam_info.pitch_limit {
                (pitch - delta_y).clamp(min_pitch, max_pitch)
            } else {
                pitch - delta_y
            };

            // Apply rotation after limit set
            cam_transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);
        }

        // Update camera rotation bvased on zoom - Also applies zoom.
        let rot_matrix = Mat3::from_quat(cam_transform.rotation);
        cam_transform.translation = rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, cam_info.zoom.radius));
    }
}

// fn sync_player_camera(
//     player_q: Query<&Transform, (With<Predicted>, With<Controlled>)>,
//     mut cam_q: Query<(&mut CamInfo, &mut Transform), Without<Predicted>>,
// ) {
//     if let Ok(player_transform) = player_q.get_single() {
//         let (cam, mut cam_transform) = cam_q.get_single_mut().expect("Camera to exist");

//         let rotation_matrix = Mat3::from_quat(cam_transform.rotation);

//         // Offset actually
//         let offset = rotation_matrix.mul_vec3(Vec3::new(0.0, 0.5, cam.zoom.radius));

//         // Update the camera translation
//         cam_transform.translation = offset + player_transform.translation;
//     }
// }
