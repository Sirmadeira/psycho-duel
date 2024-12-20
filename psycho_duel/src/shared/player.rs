use bevy::prelude::*;
use lightyear::prelude::{client::Predicted, ReplicationTarget};

use super::protocol::{PlayerActions, PlayerMarker};

/// Centralization plugin - Responsible for having player shared logic among client and server
pub struct SharedPlayerPlugin;

impl Plugin for SharedPlayerPlugin {
    fn build(&self, app: &mut App) {
        // Update because it should listen for it all the time
        app.add_systems(Update, insert_input_map);
    }
}

/// Responsible for adding key component for player movement
/// Why not use protocol here?
fn insert_input_map(
    query_players: Query<
        Entity,
        (
            Or<(Added<Predicted>, Added<ReplicationTarget>)>,
            With<PlayerMarker>,
        ),
    >,
    mut commands: Commands,
) {
    for player in query_players.iter() {
        info!("Inserting map");
        commands
            .entity(player)
            .insert(PlayerActions::default_input_map());
    }
}
