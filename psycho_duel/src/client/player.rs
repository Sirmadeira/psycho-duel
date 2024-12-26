use super::{
    egui::{ChangeCharEvent, Parts},
    load_assets::GltfCollection,
    protocol::{PlayerId, PlayerMarker, PlayerVisuals},
    ClientAppState,
};
use crate::shared::protocol::*;
use crate::shared::CommonChannel;
use bevy::animation::AnimationTarget;
use bevy::{prelude::*, utils::HashMap};
use leafwing_input_manager::prelude::*;
use lightyear::{client::prediction::Predicted, shared::events::components::MessageEvent};
use lightyear::{prelude::*, shared::replication::components::Controlled};
use std::collections::VecDeque;

/// Centralization plugin - Everything correlated to player shall be inserted here
pub struct ClientPlayerPlugin;

/// Essential plugin to be able to find player predicted entity via client_id, really usefull for event consumption
/// And to grab easily your predicted player
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct ClientIdPlayerMap {
    map: HashMap<ClientId, Entity>,
}

/// A tuple map, that intakes clientid + file path string, via him we have an easy acessor that let us despawn easily our spawned scenes,
/// when we receive the changechar event I dont want to play with lifelines in bevy it is insanely annoying that is why string
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct BodyPartMap {
    pub map: HashMap<(ClientId, String), Entity>,
}

/// Event send everytime we spawn a visual scene
#[derive(Event, Reflect)]
struct TranferAnim {
    /// Tell me the id that need to transfer anim
    id: ClientId,
    /// Exact part name that I need to transfer too
    part_name: String,
}

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        // Init resources
        app.init_resource::<ClientIdPlayerMap>();
        app.init_resource::<BodyPartMap>();

        // Init events
        app.add_event::<TranferAnim>();

        // Update added systems, should only occur when we enter state in game. Any other way doesnt make sense currrently
        app.add_systems(
            Update,
            (fill_client_id_map, render_predicted_player).run_if(in_state(ClientAppState::Game)),
        );

        // In observe because ideally this should be stateless
        app.add_observer(customize_local_player);

        // In update because observer tends to be unstable (adds component in a disorderly fashion therefore it doesnt run sometimes)
        app.add_systems(Update, customize_player_on_other_clients);

        // Transfering information from one bone to another
        app.add_systems(PostUpdate, transfer_anim_info);

        // In update because we wanna keep checking this all the time when we do lobbies
        app.add_systems(Update, insert_input_map);

        // Fixed update because input systems should be frame unrelated
        app.add_systems(FixedUpdate, move_player);

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
    mut transfer_anim_writer: EventWriter<TranferAnim>,
    mut commands: Commands,
) {
    for (parent, player_id, player_visuals) in player.iter() {
        // Avoids error - https://bevyengine.org/learn/errors/b0004/ - Dont worry, about transform here server can easily override if necessary
        commands
            .entity(parent)
            .insert(Transform::default())
            .insert(Visibility::default());
        for item in player_visuals.iter_visuals() {
            if let Some(entity) = spawn_visual_scene(item, &gltf_collection, &gltfs, &mut commands)
            {
                commands.entity(entity).set_parent(parent);
                body_part_map
                    .map
                    .insert((player_id.id, item.file_path.clone()), entity);
            }
            transfer_anim_writer.send(TranferAnim {
                id: player_id.id.clone(),
                part_name: item.name.to_string(),
            });
        }
    }
}

/// Callable function - Necessary that intakes a given a file_path string and spawns the given scene for it.
fn spawn_visual_scene(
    item: &Item,
    gltf_collection: &Res<GltfCollection>,
    gltfs: &Res<Assets<Gltf>>,
    commands: &mut Commands,
) -> Option<Entity> {
    // Grab file path and name from our visual item and spawn him
    if let Some(gltf) = gltf_collection.gltf_files.get(&item.file_path) {
        if let Some(loaded_gltf) = gltfs.get(gltf) {
            let scene = loaded_gltf.scenes[0].clone_weak();
            let id = commands
                .spawn(SceneRoot(scene))
                .insert(item.name.clone())
                .id();
            Some(id)
        } else {
            warn!("Didnt manage to find gltf handle in our gltf asset");
            None
        }
    } else {
        warn!(
            "Couldnt find the given file path {} in our gltf collection did you forget to add him?",
            item
        );
        None
    }
}

/// This guy is quite a biggie so here is a full explanation on how our character customizer works
/// -> First either egui or UI shall send a change char event message to client
/// -> THe function customize_local_player shall consume this local event and do all the actions necessary to make the customization occurs
/// -> After that we send a message to server, a save message, server shall validate it
/// -> If he okays, he will change the confirmed entity, and propagate the save message to the other clients
/// -> If not he will tell that client, he can test it but when he leaves he will enter a rollback state where we reverse him.
/// -> Why save message? Well because indepently of what happens we will have to save the entire save, might as well make that clear.
/// -> Why like this? Well to ensure no visual hacks and also to let player test out visuals he doesnt have access to.
/// -> Why predicted player? Well because we solely want to change predicted entities via client, confirmed are the ones altered by server!
fn customize_local_player(
    change_char: Trigger<ChangeCharEvent>,
    mut player_visuals: Query<&mut PlayerVisuals, With<Predicted>>,
    player_map: Res<ClientIdPlayerMap>,
    mut body_part_map: ResMut<BodyPartMap>,
    gltf_collection: Res<GltfCollection>,
    gltfs: Res<Assets<Gltf>>,
    mut connection_manager: ResMut<ClientConnectionManager>,
    mut transfer_anim_writer: EventWriter<TranferAnim>,
    mut commands: Commands,
) {
    let event = change_char.event();

    let client_id = &event.client_id;
    let new_item = &event.item;
    let body_part = &event.body_part;

    customize_player(
        client_id,
        body_part,
        new_item,
        &mut player_visuals,
        &player_map,
        &mut body_part_map,
        &gltf_collection,
        &gltfs,
        &mut commands,
    );

    if connection_manager
        .send_message::<CommonChannel, SaveMessage>(&mut SaveMessage {
            id: *client_id,
            change_char: Some(event.clone()),
            change_currency: None,
            change_inventory: None,
        })
        .is_err()
    {
        warn!("Failed to send save to server!")
    }
    transfer_anim_writer.send(TranferAnim {
        id: client_id.clone(),
        part_name: new_item.name.to_string(),
    });
}

/// Validates if server gave the okay or not, after that we customize our other clients
/// Only applies customization logic, even tho it captures save message
fn customize_player_on_other_clients(
    mut save_message: EventReader<MessageEvent<SaveMessage>>,
    mut player_visuals: Query<&mut PlayerVisuals, With<Predicted>>,
    player_map: Res<ClientIdPlayerMap>,
    mut body_part_map: ResMut<BodyPartMap>,
    opt_gltf_collection: Option<Res<GltfCollection>>,
    gltfs: Res<Assets<Gltf>>,
    mut transfer_anim_writer: EventWriter<TranferAnim>,
    mut commands: Commands,
) {
    for event in save_message.read() {
        let message = event.message();
        if let Some(change_char) = &message.change_char {
            info!("Server gave the okay lets change this client on others");
            let client_id = &message.id;
            let body_part = &change_char.body_part;
            let new_item = &change_char.item;

            if let Some(gltf_collection) = &opt_gltf_collection {
                customize_player(
                    client_id,
                    body_part,
                    new_item,
                    &mut player_visuals,
                    &player_map,
                    &mut body_part_map,
                    &gltf_collection,
                    &gltfs,
                    &mut commands,
                );
                transfer_anim_writer.send(TranferAnim {
                    id: client_id.clone(),
                    part_name: new_item.name.to_string(),
                });
            } else {
                warn!("This client is most probably in a loading state")
            }
        }
    }
}

fn customize_player(
    client_id: &ClientId,
    body_part: &Parts,
    new_item: &Item,
    player_visuals: &mut Query<&mut PlayerVisuals, With<Predicted>>,
    player_map: &Res<ClientIdPlayerMap>,
    body_part_map: &mut ResMut<BodyPartMap>,
    gltf_collection: &Res<GltfCollection>,
    gltfs: &Res<Assets<Gltf>>,
    mut commands: &mut Commands,
) {
    // Finding player entity via map
    if let Some(entity) = player_map.map.get(client_id) {
        // Grab the player's current visuals not mutably server needs to validate
        let player_visual = player_visuals
            .get(*entity)
            .expect("Player to be online and to have visual component");

        // Determine the current part
        let current_item = player_visual.get_visual(body_part);
        let curr_file_path = &current_item.file_path;
        let new_file_path = &new_item.file_path;

        // Only proceed if the new part is different from current in player visual
        if curr_file_path != new_file_path {
            info!(
                "Changed {:?} visual item for client {:?} from '{}' to '{}'",
                body_part, client_id, curr_file_path, new_file_path
            );

            // Removing old part from the map
            let key = (*client_id, curr_file_path.to_string());
            if let Some(entity) = body_part_map.map.remove(&key) {
                info!("Removing old body part from entity {}", entity);
                commands.entity(entity).despawn_recursive();
            }

            // Spawn new visual scene and insert into the map
            if let Some(id) = spawn_visual_scene(&new_item, &gltf_collection, &gltfs, &mut commands)
            {
                info!("Spawning new visual scene for {}", client_id);
                body_part_map
                    .map
                    .insert((*client_id, new_file_path.to_string()), id);

                // Make player parent of the new spawned scene
                commands.entity(id).set_parent(*entity);
            } else {
                panic!("Something went wrong spawning new visual scene");
            }
        } else {
            info!(
                "Visual item '{}' for client {} is already current; no changes made",
                curr_file_path, client_id
            );
        }
    } else {
        warn!(
            "Something went terribly wrong; couldn't find client_id {} player entity",
            client_id
        );
    }
}

/// Grabs player skeleton item, from him transfers all of his animation player and targets_id to visual bones, which are not animated
fn transfer_anim_info(
    mut transfer_anim: EventReader<TranferAnim>,
    player_map: Res<ClientIdPlayerMap>,
    children: Query<&Children>,
    names: Query<&Name>,
    animation_targets: Query<&AnimationTarget>,
    mut commands: Commands,
) {
    for transfer in transfer_anim.read() {
        let client_id = transfer.id;
        let part_name = &transfer.part_name;

        // Skeleton does not need to transfer animation as he already carries our animatoions
        if part_name.contains("skeleton") {
            return;
        }

        // Unwraps because it is pretty much impossible for predicted player to not be on map or skeleton to not be a child when triggering this guy
        let player_ent = player_map.map.get(&client_id).unwrap();

        //  Find player skeleton
        let old_skeleton =
            find_child_with_name_containing(&children, &names, player_ent, "skeleton").unwrap();

        // Find his armature - First animation player is usually here
        let old_armature =
            find_child_with_name_containing(&children, &names, &old_skeleton, "Armature").unwrap();

        // Insert his bones into a map
        let mut old_bones = HashMap::new();
        collect_bones(&children, &names, &old_armature, &mut old_bones);

        // Part that needs to have it is children targets killed
        let new_part =
            find_child_with_name_containing(&children, &names, player_ent, &part_name).unwrap();

        // Finding her animation player carries
        let new_armature =
            find_child_with_name_containing(&children, &names, &new_part, "Armature").unwrap();

        // Insert new animation player version
        commands
            .entity(new_armature)
            .insert(AnimationPlayer::default());

        let mut new_bones = HashMap::new();
        collect_bones(&children, &names, &new_armature, &mut new_bones);

        // Tranfering animation targets
        for (name, old_bone) in old_bones.iter() {
            let old_animation_target = animation_targets
                .get(*old_bone)
                .expect("To have target if it doesnt well shit");

            if let Some(corresponding_bone) = new_bones.get(name) {
                commands
                    .entity(*corresponding_bone)
                    .insert(AnimationTarget {
                        id: old_animation_target.id,
                        player: new_armature,
                    });
            }
        }
    }
}

/// Collect all sub children bones of that specific part, usually starts by the armature bone
fn collect_bones(
    children_entities: &Query<&Children>,
    names: &Query<&Name>,
    root_bone: &Entity,
    collected: &mut HashMap<String, Entity>,
) {
    if let Ok(name) = names.get(*root_bone) {
        collected.insert(format!("{}", name), *root_bone);

        if let Ok(children) = children_entities.get(*root_bone) {
            for child in children {
                collect_bones(children_entities, names, child, collected)
            }
        }
    }
}

/// Helper Finds a bone with a certain name, from what I could see this is the most optimal way
fn find_child_with_name_containing(
    children: &Query<&Children>,
    names: &Query<&Name>,
    entity: &Entity,
    name_to_match: &str,
) -> Option<Entity> {
    let mut queue = VecDeque::new();
    queue.push_back(entity);

    while let Some(curr_entity) = queue.pop_front() {
        let name_result = names.get(*curr_entity);
        if let Ok(name) = name_result {
            if format!("{}", name).contains(name_to_match) {
                // found the named entity
                return Some(*curr_entity);
            }
        }

        let children_result = children.get(*curr_entity);
        if let Ok(children) = children_result {
            for child in children {
                queue.push_back(child)
            }
        }
    }

    return None;
}

/// Whenever we get a predicted entity that is controlled we add the input map unto it
fn insert_input_map(
    query: Query<(Entity, Has<Controlled>), Added<Predicted>>,
    mut commands: Commands,
) {
    for (entity, is_controlled) in query.iter() {
        if is_controlled {
            commands
                .entity(entity)
                .insert(PlayerActions::default_input_map());
        }
    }
}

/// Move player solely moves the predicted controlled player, later server will also give you the true position
fn move_player(
    mut player_action: Query<(&ActionState<PlayerActions>, &mut Transform), With<Predicted>>,
) {
    for (player_action, mut transform) in player_action.iter_mut() {
        // You know only act when we actually have something to do
        if !player_action.get_pressed().is_empty() {
            // Make this shared
            if player_action.pressed(&PlayerActions::Forward) {
                transform.translation += Vec3::new(0.0, 0.0, 0.1);
            }
            if player_action.pressed(&PlayerActions::Backward) {
                transform.translation -= Vec3::new(0.0, 0.0, 0.1);
            }
            if player_action.pressed(&PlayerActions::Left) {
                transform.translation += Vec3::new(0.1, 0.0, 0.0);
            }
            if player_action.pressed(&PlayerActions::Right) {
                transform.translation -= Vec3::new(0.1, 0.0, 0.0);
            }
        }
    }
}
