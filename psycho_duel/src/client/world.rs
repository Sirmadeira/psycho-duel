use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, render::view::NoFrustumCulling};

use super::protocol::{CycleTimer, SunMarker};

/// Orbit around what
const SUN_ORBIT_AROUND: Vec3 = Vec3::ZERO;

/// Radius of our sun orbit, usually we want this guy to be a biggie for reasons
const SUN_RADIUS: f32 = 15.0;

/// Min iluminance we dont want our character to be basically black
const MIN_ILUMINANCE: f32 = 400.0;

/// Max iluminancer we dont want our character to be basically white
const MAX_ILUMINANCE: f32 = 10000.0;

/// Centralization plugin - World building logic should be here
pub struct ClientWorldPlugin;

impl Plugin for ClientWorldPlugin {
    fn build(&self, app: &mut App) {
        // Update because we need to check when sun gets spawned
        app.add_systems(Update, formulate_client_sun);
        app.add_systems(FixedUpdate, orbit_around_point);
    }
}

/// Add extra components to our sun exclusive to our client things like directional light. Transform and such
fn formulate_client_sun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sun_q: Query<Entity, Added<SunMarker>>,
) {
    if let Ok(sun) = sun_q.get_single() {
        commands
            .entity(sun)
            .insert(DirectionalLight {
                illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                shadows_enabled: true,
                ..default()
            })
            .insert(
                CascadeShadowConfigBuilder {
                    first_cascade_far_bound: 2.0,
                    maximum_distance: 20.0,
                    ..default()
                }
                .build(),
            )
            .insert(Transform::default())
            // To ensure that even with the light source still leaves camera we render it
            .insert(NoFrustumCulling)
            .insert(Mesh3d(meshes.add(Cuboid::default())))
            .insert(MeshMaterial3d(materials.add(StandardMaterial::default())));
    }
}

/// Creates the orbit of our sun he start from sin the beginning of the trigonometric circle. Sin equal 0 cosine equal 1.
/// AS he move to the peak he increases sine which directly increases our light.
fn orbit_around_point(
    mut query: Query<(&mut Transform, &mut DirectionalLight, &CycleTimer), With<SunMarker>>,
) {
    for (mut transform, mut directional_light, cycle_time) in query.iter_mut() {
        // Calculates the amount of cycle that has passed
        let cycle_fraction =
            cycle_time.cycle.elapsed_secs() / cycle_time.cycle.duration().as_secs_f32();

        // Calculate the angle fraction of the orbit
        let angle = cycle_fraction * std::f32::consts::PI * 2.0;

        // Calculate the new target position using trigonometric functions
        let target_position = Vec3::new(
            -SUN_ORBIT_AROUND.x + SUN_RADIUS * angle.cos(),
            SUN_ORBIT_AROUND.y + SUN_RADIUS * angle.sin(),
            0.0,
        );

        // Multiple update ticks might occur before the replicated value actually appears, which means we can just interpolate without the whole previous
        // Values debacle, we could also extrapolate here as we know for certain the cycle is gonna be but since time is very slow in the game. I simply dont care lawrence
        transform.translation = transform.translation.lerp(target_position, 0.1);

        transform.look_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);

        // You probably wondering why sine? Well we start from 0 also know as like or 0 radians.
        // Cosine when angle is 0 is 1, as he starts to rotate we go from 1 to 0 to -1 than to 0 than to 1
        // When he gets to minus one we stop diminishing because there is simply no reason to keep going
        let target_illuminance = if angle.sin() >= 0.0 {
            // Calculate the target illuminance based on the angle
            MIN_ILUMINANCE + (MAX_ILUMINANCE - MIN_ILUMINANCE) * angle.sin()
        } else {
            0.02
        };

        // Smoothly interpolate current illuminance to the target
        directional_light.illuminance = directional_light.illuminance.lerp(target_illuminance, 0.1);
    }
}
