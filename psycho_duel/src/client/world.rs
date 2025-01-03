use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, render::view::NoFrustumCulling};

use super::protocol::{CycleTimer, SunMarker};

/// Orbit around what
const SUN_ORBIT_AROUND: Vec3 = Vec3::ZERO;

/// Radius of our sun orbit, usually we want this guy to be a biggie for reasons
const SUN_RADIUS: f32 = 50.0;

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
        // Update because it receives server replication and we need to interpolate to that fixed point between our ticks
        app.add_systems(Update, orbit_around_point);
        // Spawn our white space also know as player house
        app.add_systems(Startup, spawn_white_space);
        // Spawns our spotlight that ensures that player see our figure
        app.add_systems(Startup, spawn_spotlight);
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
            MIN_ILUMINANCE
        };

        // Smoothly interpolate current illuminance to the target
        directional_light.illuminance = directional_light.illuminance.lerp(target_illuminance, 0.1);
    }
}

/// Meshes tend to have the necessity of being stored into handles. Why? Well because on later notices if you want to use it
/// You have an easy way of spawming the same mesh
fn spawn_white_space(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)));
    let floor_material = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.98, 1.0),
        metallic: 0.3, // High metallic for reflectivity
        perceptual_roughness: 0.05,
        reflectance: 0.8,
        ..default()
    }));

    // Spawning base floor
    commands.spawn(Mesh3d(floor.clone())).insert(floor_material);
}

// TODO SPOTLIGHT AT CENTER OF PLANE POINTING TO FIGURE
fn spawn_spotlight(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let spotlight = SpotLight {
        intensity: 100000000.0,
        color: Color::WHITE,
        range: 40.0,
        // If you insert this you get a cool visual like a light inside a light
        radius: 0.0,
        shadows_enabled: true,
        inner_angle: std::f32::consts::PI / 4.0 * 0.85,
        outer_angle: std::f32::consts::PI / 4.0,
        ..default()
    };

    // This is gonna be looking right below him
    commands
        .spawn(spotlight)
        .insert(Name::new("FigureLight"))
        .insert(Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::new(0.0, 0.0, 20.0), Vec3::Y))
        .insert(Mesh3d(meshes.add(Cuboid::default())))
        .insert(MeshMaterial3d(materials.add(StandardMaterial::default())))
        .insert(NoFrustumCulling);
}
