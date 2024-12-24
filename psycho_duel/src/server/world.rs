use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use super::protocol::{CycleTimer, SunMarker};

/// Centralization plugin - World building logic should be here
pub struct ServerWorldPlugin;

impl Plugin for ServerWorldPlugin {
    fn build(&self, app: &mut App) {
        // In startup because it should occur imediatelly
        app.add_systems(Startup, spawn_sun);
        // In FixedUpdate to ensure that a server frame rate doesnt actually influence how fast the sun orbits
        app.add_systems(FixedUpdate, tick_orbit_cycle);
    }
}

/// Spawn our orbiting sun entity that is constantly replicated to client
fn spawn_sun(mut commands: Commands) {
    let replicate = Replicate {
        target: ReplicationTarget {
            target: NetworkTarget::All,
        },
        ..default()
    };
    commands
        .spawn(SunMarker)
        .insert(replicate)
        .insert(CycleTimer::default())
        .insert(Name::new("Sun"));
}

fn tick_orbit_cycle(mut sun_q: Query<&mut CycleTimer, With<SunMarker>>) {
    if let Ok(mut cycle_timer) = sun_q.get_single_mut() {
        cycle_timer.cycle.tick(Duration::from_secs(1));
    }
}
