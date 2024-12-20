use std::ops::DerefMut;

use crate::server::ClientId;
use crate::shared::protocol::PlayerId;
use bevy::prelude::*;
use bevy::transform::commands;
use bevy::utils::HashMap;
use lightyear::prelude::server::{ControlledBy, Lifetime, Replicate, SyncTarget};
use lightyear::prelude::*;
use lightyear::server::events::DisconnectEvent;

use super::protocol::{CoreInformation, PlayerMarker};

/// Simple map - That points out the player entity with that given id
/// Pass a client_id get it is server player entity
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ServerClientIdPlayerMap {
    pub map: HashMap<ClientId, Entity>,
}

/// Centralization plugin - Utilized for player related logic
pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerClientIdPlayerMap>();

        // Observes when core inserted
        app.observe(spawn_player_when_core);
        // Observer when player is created
        app.observe(add_initial_position);

        // In update because it is an event listener
        app.add_systems(Update, despawns_player_when_disconnects);
    }
}

/// Whenever a core information exists spawn player entity with a given local position
/// For now it increments
fn spawn_player_when_core(
    core: Trigger<OnAdd, CoreInformation>,
    player_ids: Query<&PlayerId>,
    mut player_map: ResMut<ServerClientIdPlayerMap>,
    mut commands: Commands,
) {
    let core_entity = core.entity();
    if let Ok(player_id) = player_ids.get(core_entity) {
        let client_id = player_id.id;
        let player = formulates_player(&client_id, core_entity, &mut commands);

        player_map.map.insert(client_id, player);
        info!(
            "Received core information trigger spawning new player for id and adding him to map {}",
            client_id
        )
    }
}

/// Callable function - Responsible for adding additional non optional player fields into core entity
fn formulates_player(client_id: &ClientId, entity: Entity, commands: &mut Commands) -> Entity {
    // Okay here is a quick explanation of this guy - He
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
        .entity(entity)
        .insert(PlayerMarker)
        .insert(Name::new(format!("Player {}", client_id)))
        .insert(replicate)
        .id();
    id
}

/// Adjust player so he is slightly ahead of another
fn add_initial_position(
    player: Trigger<OnAdd, PlayerMarker>,
    mut offset: Local<f32>,
    mut commands: Commands,
) {
    let player_ent = player.entity();
    commands
        .entity(player_ent)
        .insert(Transform::from_translation(Vec3::new(0.0, 0.0, *offset)));

    *offset += 0.5;
}

/// Currently we are despawning players, whenever an disconnect occurs
/// COOL TODO - Visually display when a player rage quitted in game. Added in issues https://github.com/Sirmadeira/psycho-duel/issues/2
fn despawns_player_when_disconnects(
    mut disconnection: EventReader<DisconnectEvent>,
    mut player_map: ResMut<ServerClientIdPlayerMap>,
    mut commands: Commands,
) {
    for event in disconnection.read() {
        let client_id = event.client_id;

        if let Some(entity) = player_map.map.remove(&client_id) {
            info!("Despawning player entity {} for {}", entity, client_id);
            commands.entity(entity).despawn_recursive();
        } else {
            warn!("Something is wrong with player despawning, couldnt manage to find this client entity {}",client_id);
        }
    }
}
