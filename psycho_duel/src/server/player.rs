use crate::server::ClientId;
use crate::shared::protocol::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

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
        app.add_observer(spawn_player_when_core);
        // Observer when player is created
        app.add_observer(add_initial_position);

        // Update because we wanna check for that consantly
        app.add_systems(Update, insert_input_map);

        // Wait until you receive the newest player input message before you actually replicate to the others
        app.add_systems(PreUpdate, replicate_inputs.after(MainSet::EmitEvents));

        // Fixed update becaue movement should be frame unrelated
        app.add_systems(FixedUpdate, move_player);

        // In update because it is an event listener
        app.add_systems(Update, despawns_player_when_disconnects);

        //Debug
        app.register_type::<ServerClientIdPlayerMap>();
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
        group: REPLICATION_GROUP,
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
    mut disconnection: EventReader<ServerDisconnectEvent>,
    mut player_map: ResMut<ServerClientIdPlayerMap>,
    mut commands: Commands,
) {
    for event in disconnection.read() {
        let client_id = event.client_id;
        info!("Despawning player entity for {}", client_id);
        if let Some(entity) = player_map.map.remove(&client_id) {
            commands.entity(entity).despawn_recursive();
        } else {
            warn!("Something is wrong with player despawning, couldnt manage to find this client entity {}",client_id);
        }
    }
}

/// Whenever we get a player marker entity we add the input map unto it
fn insert_input_map(query: Query<Entity, Added<PlayerMarker>>, mut commands: Commands) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(PlayerActions::default_input_map());
    }
}

/// After receiveing action state via input message we replicate that client action to the other clients
/// So we guarantee that they can be predicted
fn replicate_inputs(
    mut connection: ResMut<ServerConnectionManager>,
    mut input_events: ResMut<Events<MessageEvent<InputMessage<PlayerActions>>>>,
) {
    for mut event in input_events.drain() {
        let client_id = *event.context();

        // Optional: do some validation on the inputs to check that there's no cheating
        // Inputs for a specific tick should be write *once*. Don't let players change old inputs.

        // rebroadcast the input to other clients
        connection
            .send_message_to_target::<InputChannel, _>(
                &mut event.message,
                NetworkTarget::AllExceptSingle(client_id),
            )
            .unwrap()
    }
}

/// When player action is active - Do action
fn move_player(mut player_action: Query<(&ActionState<PlayerActions>, &mut Transform)>) {
    for (player_action, mut transform) in player_action.iter_mut() {
        if !player_action.get_pressed().is_empty() {
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
