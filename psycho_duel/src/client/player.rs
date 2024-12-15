use super::{
    egui::{ChangeCharEvent, Parts},
    load_assets::GltfCollection,
    protocol::{PlayerId, PlayerMarker, PlayerVisuals},
    ClientAppState,
};
use crate::shared::protocol::*;
use crate::shared::CommonChannel;
use bevy::{prelude::*, utils::HashMap};
use lightyear::prelude::*;
use lightyear::{client::prediction::Predicted, shared::events::components::MessageEvent};
/// Centralization plugin - Everything correlated to player shall be inserted here
pub struct ClientPlayerPlugin;

/// Essential plugin to be able to find player predicted entity via client_id, really usefull for event consumption
/// And to grab easily your predicted player
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

        // In observe because ideally this should be stateless
        app.observe(customize_local_player);

        // In update because we need to check this continously
        app.add_systems(Update, customize_player_on_other_clients);

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

/// Callable function - Necessary that intakes a given a file_path string and spawns the given scene for it.
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

/// This guy is quite a biggie so here is a full explanation on how our character customizer works
/// -> First either egui or UI shall send a change char event message to client
/// -> THe function customize_player shall consume this local event and do all the actions necessary to make the customization occurs
/// -> After that we send a message to server, server shall validate if the character can leave the customization screen
/// -> If he okays it he will send a save message event with an optional field change char filled
/// -> If not the optional field will not be filled, when that happens
/// -> That client will enter a rollback state where when he leaves the current ui state, his character will automatically return to the previous visual state
/// -> Why like this? Well to ensure no visual hacks and also to let player test out visuals he doesnt have access to.
fn customize_local_player(
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
    let customized_visual = customize_player(
        client_id,
        body_part,
        part_to_change,
        &mut player_visuals,
        &player_map,
        &mut body_part_map,
        &gltf_collection,
        &gltfs,
        &mut commands,
    );
    if let Some(custom_visual) = customized_visual {
        if connection_manager
            .send_message::<CommonChannel, SaveMessage>(&mut SaveMessage {
                save_info: CoreInformation::total_new(*client_id, custom_visual.clone()),
                change_char: Some(event.clone()),
            })
            .is_err()
        {
            warn!("Failed to send save to server!")
        }
    }
}

/// Validates if server gave the okay or not, after that we customize our other clients
fn customize_player_on_other_clients(
    mut save_message: EventReader<MessageEvent<SaveMessage>>,
    mut player_visuals: Query<&mut PlayerVisuals, With<Predicted>>,
    player_map: Res<ClientIdPlayerMap>,
    mut body_part_map: ResMut<BodyPartMap>,
    opt_gltf_collection: Option<Res<GltfCollection>>,
    gltfs: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for event in save_message.read() {
        let message = event.message();
        //Wowzers that is a lot of fields
        let client_id = &message.save_info.player_id.id;
        if let Some(change_char) = &message.change_char {
            info!("Server gave the okay lets change this client on other");
            let body_part = &change_char.body_part;
            let part_to_change = &change_char.path_to_part;

            if let Some(gltf_collection) = &opt_gltf_collection {
                let _ = customize_player(
                    client_id,
                    body_part,
                    part_to_change,
                    &mut player_visuals,
                    &player_map,
                    &mut body_part_map,
                    &gltf_collection,
                    &gltfs,
                    &mut commands,
                );
            } else {
                warn!("This client is most probably in a loading state")
            }
        } else {
            warn!("You dont have this skin yet !")
        }
    }
}

fn customize_player(
    client_id: &ClientId,
    body_part: &Parts,
    part_to_change: &String,
    player_visuals: &mut Query<&mut PlayerVisuals, With<Predicted>>,
    player_map: &Res<ClientIdPlayerMap>,
    body_part_map: &mut ResMut<BodyPartMap>,
    gltf_collection: &Res<GltfCollection>,
    gltfs: &Res<Assets<Gltf>>,
    mut commands: &mut Commands,
) -> Option<PlayerVisuals> {
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

                // Mutating player visuals do this after every validation has been passed
                *current_part = part_to_change.clone();

                // Make player parent of the new spawned scene
                commands.entity(id).set_parent(*entity);
                Some(player_visual.clone())
            } else {
                panic!("Somethign went wrong spawning new visual scene");
            }
        } else {
            info!(
                "Part '{}' for client {} is already current; no changes made",
                part_to_change, client_id
            );
            None
        }
    } else {
        warn!(
            "Something went terribly wrong; couldn't find client_id {} player entity",
            client_id
        );
        None
    }
}
