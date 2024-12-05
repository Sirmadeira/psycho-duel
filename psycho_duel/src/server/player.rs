use crate::server::ClientId;
use crate::shared::protocol::PlayerId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use lightyear::prelude::server::{ControlledBy, Lifetime, Replicate, SyncTarget};
use lightyear::prelude::{NetworkTarget, ReplicationTarget};
use lightyear::server::events::ConnectEvent;

/// Simple map - That points out the player entity with that given id
/// Pass a client_id get it is server player entity
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ServerClientIdPlayerMap {
    map: HashMap<ClientId, Entity>,
}

/// Whenever a player connects spawn player entity
fn spawn_player_when_connects(
    mut connections: EventReader<ConnectEvent>,
    mut player_map: ResMut<ServerClientIdPlayerMap>,
    mut commands: Commands,
) {
    for event in connections.read() {
        //Reading event from lightyear that occurs whenever a client connects to my server.
        let client_id = event.client_id;
        let entity = spawn_player(&client_id, &mut commands);
        player_map.map.insert(client_id, entity);
        info!(
            "Received connect event from server spawning new player for id and adding him to map {}",
            client_id
        )
    }
}

/// Nested function - Responsible for spawning player
fn spawn_player(client_id: &ClientId, commands: &mut Commands) -> Entity {
    // This guy is very important tells us the replication type for our player entity
    let replicate = Replicate {
        target: ReplicationTarget {
            target: NetworkTarget::All,
        },
        sync: SyncTarget {
            prediction: NetworkTarget::All,
            ..default()
        },
        controlled_by: ControlledBy {
            target: NetworkTarget::Single(*client_id),
            lifetime: Lifetime::Persistent,
        },
        ..default()
    };
    //Spawning ids
    let id = commands
        .spawn(PlayerId { id: *client_id })
        .insert(Name::new(format!("Player {}", client_id)))
        .insert(replicate)
        .id();
    id
}

/// Centralization plugin - Utilized for player related logic
pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerClientIdPlayerMap>();

        // In update because it is an event listener
        app.add_systems(Update, spawn_player_when_connects);
    }
}
