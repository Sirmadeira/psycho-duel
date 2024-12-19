use crate::client::egui::ChangeCharEvent;
use crate::server::protocol::*;
use bevy::prelude::*;
use bincode::{deserialize_from, serialize_into};
use lightyear::prelude::*;
use lightyear::server::events::MessageEvent;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind};

use super::player::ServerClientIdPlayerMap;
use super::protocol::{PlayerVisuals, SaveMessage};
use super::CommonChannel;

/// Plugin utilized to store specific username info, for example: What visuals he currently has? What itens he bought? The list goes on
/// We shall store that locally via bin files while in dev.
/// In prod - This will differ a lot, this will be a queryable dataset. That we will have to consult and transform into the resource.
pub struct SavePlugin;

const SAVE_FILE_PATH: &str = "./psycho_duel/src/server/save_files/player_info.bar";

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // Init resource that is gonna be replicated
        app.init_resource::<CoreSaveInfoMap>();

        // Startup because ideally we should only run this once really early
        app.add_systems(Startup, create_or_read_save_file);

        // Update because if changes have been made we want to replicate those server changes to client
        app.add_systems(Update, replicate_resource);

        // Update because we want to keep listening to it
        app.add_systems(Update, handle_new_clients);

        // Update because it listens to client messages
        app.add_systems(Update, check_client_sent_core_information);
    }
}

/// A simple function made to ensure that everyone knows to whon we are replicating this resource to
fn replicate_resource(mut commands: Commands) {
    commands.replicate_resource::<CoreSaveInfoMap, CommonChannel>(NetworkTarget::All);
}

/// Evaluates if it is a new client or someone who has already logged in
fn handle_new_clients(
    mut save_info: ResMut<CoreSaveInfoMap>,
    mut connections: EventReader<ServerConnectEvent>,
    mut commands: Commands,
) {
    for event in connections.read() {
        let client_id = event.client_id;
        info!("Handling connect event, checking if new player or old player");

        // Check if the client already exists in the save info map
        if let Some(core_information) = save_info.map.get(&client_id) {
            info!("Old player logging in");
            // Spawn an entity with the existing core information
            commands.spawn(core_information.clone());
        } else {
            info!("New player logging in");
            // Handle a new client by creating a default core information and insert him into map
            let core_information = CoreInformation::new(client_id);
            commands.spawn(core_information.clone());
            save_info.map.insert(client_id, core_information);

            // We use references here because you know i am trying to better my clone usage
            save(&save_info);
        }
    }
}
/// A simple function that save in bincode files the adjusted resources CoreSaveInfoMap. Should occur everytime we modify that core resource in code,
/// Example: User modifies current skin, save!
fn save(save_info_map: &CoreSaveInfoMap) {
    info!("Saving new information!");
    // Unwraps here because I dont see how one would be able to just change a const or a already initialized struct field type
    let mut f = BufWriter::new(File::create(SAVE_FILE_PATH).unwrap());
    serialize_into(&mut f, &save_info_map).unwrap();
}

/// Currently this creates or reads a bincode file that stores all of our previous users infos
/// We could later make it into a NOSQL database. Not sure if necessary
fn create_or_read_save_file(mut commands: Commands) {
    // First check if we are able to open save_file path
    match File::open(SAVE_FILE_PATH) {
        // If okay we insert the replicated resource unto server
        Ok(file) => {
            info!("Managed to open pre-existing save file");
            let buf_reader = BufReader::new(file);

            let save_info: CoreSaveInfoMap = deserialize_from(buf_reader).expect(
                "If this breaks is because you changed CoreSaveInfoMap fields just delete old save folder",
            );

            commands.insert_resource(save_info);
        }
        // If not okay because we didnt found the file. We create a new file and inititialize a default value for our save resource.
        Err(err) if err.kind() == ErrorKind::NotFound => {
            info!("File doesnt currently exist creating a default CoreSaveInfoMap");
            let mut f = BufWriter::new(
                File::create(SAVE_FILE_PATH).expect("To be able to create new file"),
            );

            let save_info = CoreSaveInfoMap::default();
            serialize_into(&mut f, &save_info)
                .expect("To be able to serialize into new file on startup");

            commands.insert_resource(save_info);
        }
        Err(err) => {
            panic!("Failed to open save file for an unexpected reason: {}", err);
        }
    }
}

/// First - Check save messages optional field sent by client
/// Second - If they can occur he mutates server entity and by definition confirmed, if not, rollback mechanics - TODO
/// Third - Save new informations
fn check_client_sent_core_information(
    mut save_from_client: EventReader<MessageEvent<SaveMessage>>,
    mut core_info_map: ResMut<CoreSaveInfoMap>,
    player_map: Res<ServerClientIdPlayerMap>,
    mut player_visual: Query<&mut PlayerVisuals>,
    mut player_currency: Query<&mut Currency>,
    mut player_inventory: Query<&mut Inventory>,
    mut connection_manager: ResMut<ServerConnectionManager>,
) {
    for save_message in save_from_client.read() {
        let message = save_message.message();
        let client_id = message.id;

        if let Some(mut previous_core) = core_info_map.map.get_mut(&client_id) {
            let player_entity = player_map.map.get(&client_id).unwrap();

            // Handle visual changes
            validate_visual_change(
                &message.change_char,
                &mut previous_core,
                &mut player_visual,
                &player_inventory,
                *player_entity,
            );

            // Handle currency changes
            validate_currency_change(
                &message.change_currency,
                previous_core,
                &mut player_currency,
                *player_entity,
            );

            // Handle inventory changes
            validate_inventory_change(
                &message.change_inventory,
                previous_core,
                &mut player_inventory,
                *player_entity,
            );

            let mut message = save_message.message().clone();
            // Broadcast save message
            if connection_manager
                .send_message_to_target::<CommonChannel, SaveMessage>(
                    &mut message,
                    NetworkTarget::AllExceptSingle(client_id),
                )
                .is_err()
            {
                warn!("Even tho server gave the okay couldnt broadcast message to all clients!")
            }

            // Save core information
            save(&core_info_map);
        }
    }
}

fn validate_visual_change(
    change_char: &Option<ChangeCharEvent>,
    previous_core: &mut CoreInformation,
    player_visual: &mut Query<&mut PlayerVisuals>,
    player_inventory: &Query<&mut Inventory>,
    player_entity: Entity,
) {
    if let Some(change_visual) = change_char {
        let mut server_visual = player_visual.get_mut(player_entity).unwrap();
        let player_inventory = player_inventory.get(player_entity).unwrap();
        let body_part = &change_visual.body_part;
        let old_item = server_visual.get_visual_mut(body_part);
        let new_item = &change_visual.item;
        if player_inventory.items.get(&new_item.id).is_some() {
            *old_item = new_item.clone();
            previous_core.player_visuals = server_visual.clone();
        } else {
            *old_item = new_item.clone();
            previous_core.player_visuals = server_visual.clone();
            warn!("Validation failed: insufficient inventory. TODO: Rollback mechanic");
        }
    }
}

fn validate_currency_change(
    change_currency: &Option<Currency>,
    previous_core: &mut CoreInformation,
    player_currency: &mut Query<&mut Currency>,
    player_entity: Entity,
) {
    if let Some(currency) = change_currency {
        let mut prev_currency = player_currency.get_mut(player_entity).unwrap();
        if currency.amount < 0.0 {
            warn!("Validation failed: negative currency. TODO: Handle negative currency");
        }
        *prev_currency = *currency;
        previous_core.currency = *prev_currency;
    }
}

fn validate_inventory_change(
    change_inventory: &Option<Inventory>,
    previous_core: &mut CoreInformation,
    player_inventory: &mut Query<&mut Inventory>,
    player_entity: Entity,
) {
    if let Some(inventory) = change_inventory {
        let mut prev_inventory = player_inventory.get_mut(player_entity).unwrap();
        *prev_inventory = inventory.clone();
        previous_core.inventory = inventory.clone();
    }
}
