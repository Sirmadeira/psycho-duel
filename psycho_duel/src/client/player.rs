use super::{
    egui::ChangeCharEvent,
    load_assets::GltfCollection,
    protocol::{PlayerId, PlayerMarker, PlayerVisuals},
    ClientAppState,
};
use crate::client::egui::Parts;
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
///  when we receive the changechar event I dont want to play with lifelines in bevy it is insanely annoying that is why string
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BodyPartMap {
    pub map: HashMap<(ClientId, String), Entity>,
}

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
    mut commands: Commands,
) {
    let event = change_char.event();

    let client_id = &event.client_id;
    let part_to_change = &event.path_to_part;
    let body_part = &event.body_part;

    // Finding player entity via map
    if let Some(entity) = player_map.map.get(client_id) {
        if let Some(id) =
            spawn_visual_scene(&part_to_change, &gltf_collection, &gltfs, &mut commands)
        {
            info!("Going to spawn new visual scene for  {}", client_id);
            body_part_map
                .map
                .insert((*client_id, part_to_change.to_string()), id);
        }

        // Grab the player's current visuals mutably to update it
        let mut player_visual = player_visuals
            .get_mut(*entity)
            .expect("Player to be online and to have visual component");

        // Mutate the appropriate field based on the body part
        match body_part {
            Parts::Head => {
                let old_part = player_visual.head.clone();
                player_visual.head = part_to_change.clone();
                info!(
                    "Changed Head part for client {:?} from '{}' to '{}'",
                    client_id, old_part, part_to_change
                );
                let key = (*client_id, old_part);
                if let Some(entity) = body_part_map.map.remove(&key) {
                    info!("Removing old body part from entity {}", entity);
                    commands.entity(entity).despawn_recursive();
                }
            }
            Parts::Torso => {
                let old_part = player_visual.torso.clone();
                player_visual.torso = part_to_change.clone();
                info!(
                    "Changed Torso part for client {:?} from '{}' to '{}'",
                    client_id, old_part, part_to_change
                );
                let key = (*client_id, old_part);
                if let Some(entity) = body_part_map.map.remove(&key) {
                    info!("Removing old body part from entity {}", entity);
                    commands.entity(entity).despawn_recursive();
                }
            }
            Parts::Leg => {
                let old_part = player_visual.leg.clone();
                player_visual.leg = part_to_change.clone();
                info!(
                    "Changed Leg part for client {:?} from '{}' to '{}'",
                    client_id, old_part, part_to_change
                );
                let key = (*client_id, old_part);
                if let Some(entity) = body_part_map.map.remove(&key) {
                    info!("Removing old body part from entity {}", entity);
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    } else {
        warn!(
            "Something went terrbly wrong couldnt find client_id {} player entity",
            client_id
        )
    }
}

fn spawn_customize_scene() {}

/// TODO - Observer system check if player visuals changed in client if so tell server to adjust in all
fn notify_server_of_visual_change() {}
