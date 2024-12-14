use super::{
    egui::ChangeCharEvent,
    load_assets::GltfCollection,
    protocol::{PlayerId, PlayerMarker, PlayerVisuals},
    ClientAppState,
};
use crate::shared::protocol::*;
use crate::shared::CommonChannel;
use bevy::{prelude::*, utils::HashMap};
use lightyear::client::prediction::Predicted;
use lightyear::prelude::*;
/// Centralization plugin - Everything correlated to player shall be inserted here
pub struct ClientPlayerPlugin;

/// Essential plugin to be able to find player entity via client_id, really usefull for event consumption
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ClientIdPlayerMap {
    map: HashMap<ClientId, Entity>,
}

/// A tuple map, that intakes clientid + file path string, via him we have an easy acessor that let us despawn easily our spawned scenes,
/// when we receive the changechar event I dont want to play with lifelines in bevy it is insanely annoying that is why string
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BodyPartMap {
    pub map: HashMap<(ClientId, String), Entity>,
}

/// Event only executed once we manage to mutate the player visual field - Wait 0.15
// #[derive(Event)]
// pub struct TransferAnimations;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        // Init resources
        app.init_resource::<ClientIdPlayerMap>();
        app.init_resource::<BodyPartMap>();

        // Update added systems, should only occur when we enter state in game. Any other way doesnt make sense currrently
        app.add_systems(
            Update,
            (fill_client_id_map, render_predicted_player).run_if(in_state(ClientAppState::Game)),
        );

        // In update because we need to read events, and the order doesnt currently matter
        app.observe(customize_predicted_player);

        // Debug
        app.register_type::<ClientIdPlayerMap>();
        app.register_type::<BodyPartMap>();
    }
}

/// Whenever we have a player marker, we shal check if she is predicted if so
/// Map that entity
fn fill_client_id_map(
    player_comp: Query<(Entity, &PlayerId), (Added<Predicted>, With<PlayerMarker>)>,
    mut entity_map: ResMut<ClientIdPlayerMap>,
) {
    for (entity, player_comp) in player_comp.iter() {
        info!("Adding player entity {} unto client id map", entity);

        entity_map.map.insert(player_comp.id, entity);
    }
}

/// Whenever we spawn an entity with player visuals, we are going to check if she is predicted if so.
/// We are going to spawn their given scenes.
/// IMPORTANT - Observers are essential here, because them we dont need to worry, about resource management.
fn render_predicted_player(
    player: Query<(Entity, &PlayerId, &PlayerVisuals), Added<Predicted>>,
    mut body_part_map: ResMut<BodyPartMap>,
    gltf_collection: Res<GltfCollection>,
    gltfs: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for (parent, player_id, player_visuals) in player.iter() {
        // Avoids error - https://bevyengine.org/learn/errors/b0004/
        commands.entity(parent).insert(SpatialBundle::default());
        for file_path in player_visuals.iter_visuals() {
            if let Some(entity) =
                spawn_visual_scene(file_path, &gltf_collection, &gltfs, &mut commands)
            {
                info!(
                    "Spawning visuals for client_id {},{}, entity {} and making them children from predicted entity",
                    player_id.id, file_path, entity
                );
                commands.entity(entity).set_parent(parent);
                info!("Filling body part map with new parts and client id");
                body_part_map
                    .map
                    .insert((player_id.id, file_path.to_string()), entity);
            }
        }
    }
}

/// Nested function - Necessary that intakes a given a file_path string and spawns the given scene for it.
fn spawn_visual_scene(
    file_path: &str,
    gltf_collection: &Res<GltfCollection>,
    gltfs: &Res<Assets<Gltf>>,
    commands: &mut Commands,
) -> Option<Entity> {
    if let Some(gltf) = gltf_collection.gltf_files.get(file_path) {
        if let Some(loaded_gltf) = gltfs.get(gltf) {
            // The name will always be the last part of the file path to string
            let sliced_str = file_path
                .split("/")
                .last()
                .unwrap_or(&file_path)
                .to_string();

            let scene = loaded_gltf.scenes[0].clone_weak();
            let id = commands
                .spawn(SceneBundle {
                    scene: scene,

                    ..default()
                })
                .insert(Name::new(sliced_str))
                .id();
            Some(id)
        } else {
            warn!("Didnt manage to find gltf handle in our gltf asset");
            None
        }
    } else {
        warn!(
            "Couldnt find the given file path {} in our gltf collection did you forget to add him?",
            file_path
        );
        None
    }
}

/// Intake change char customizer event and start the given actions for rendered predicted entities
/// Notice here how a lot of things here are in if let some statements this is purposefull as later this will facilitate our migration to
/// Incremental customization and diminishing customization
fn customize_predicted_player(
    change_char: Trigger<ChangeCharEvent>,
    mut player_visuals: Query<&mut PlayerVisuals, With<Predicted>>,
    player_map: Res<ClientIdPlayerMap>,
    mut body_part_map: ResMut<BodyPartMap>,
    gltf_collection: Res<GltfCollection>,
    gltfs: Res<Assets<Gltf>>,
    mut connection_manager: ResMut<ClientConnectionManager>,
    mut commands: Commands,
) {
    let event = change_char.event();

    let client_id = &event.client_id;
    let part_to_change = &event.path_to_part;
    let body_part = &event.body_part;

    // Finding player entity via map
    if let Some(entity) = player_map.map.get(client_id) {
        // Grab the player's current visuals not mutably server needs to validate
        let mut player_visual = player_visuals
            .get_mut(*entity)
            .expect("Player to be online and to have visual component");

        // Determine the current part
        let current_part = player_visual.get_visual_mut(body_part);

        // Only proceed if the new part is different from current in player visual
        if current_part != part_to_change {
            info!(
                "Changed {:?} part for client {:?} from '{}' to '{}'",
                body_part, client_id, current_part, part_to_change
            );

            // Removing old part from the map
            let key = (*client_id, current_part.to_string());
            if let Some(entity) = body_part_map.map.remove(&key) {
                info!("Removing old body part from entity {}", entity);
                commands.entity(entity).despawn_recursive();
            }

            // Spawn new visual scene and insert into the map
            if let Some(id) =
                spawn_visual_scene(&part_to_change, &gltf_collection, &gltfs, &mut commands)
            {
                info!("Spawning new visual scene for {}", client_id);
                body_part_map
                    .map
                    .insert((*client_id, part_to_change.to_string()), id);

                // // Mutating player visuals do this after every validation has been passed
                *current_part = part_to_change.clone();

                // Make player parent of the new spawned scene
                commands.entity(id).set_parent(*entity);

                // TODO - Capture event in server, and homolog it
                if connection_manager
                    .send_message::<CommonChannel, SaveMessage>(&mut SaveMessage {
                        save_info: CoreInformation::total_new(*client_id, player_visual.clone()),
                    })
                    .is_err()
                {
                    warn!("Failed to send save to server!")
                }
            }
        } else {
            info!(
                "Part '{}' for client {} is already current; no changes made",
                part_to_change, client_id
            );
        }
    } else {
        warn!(
            "Something went terribly wrong; couldn't find client_id {} player entity",
            client_id
        )
    }
}

// /// This should only occur once everything is correct in client
// fn notify_server_of_visual_change(
//     player_comp: Query<&PlayerVisuals, Changed<PlayerVisuals>>,
//     mut connection_manager: ResMut<ClientConnectionManager>,
// ) {
//     for player_visuals in player_comp.iter() {
//         if connection_manager
//             .send_message::<CommonChannel, SaveVisual>(&mut SaveVisual {
//                 loaded_visuals: player_visuals.clone(),
//             })
//             .is_err()
//         {
//             warn!("Failed to send visual customization message to the server");
//         }
//     }
// }
