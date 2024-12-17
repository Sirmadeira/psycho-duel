use crate::server::protocol::CoreInformation;
use crate::shared::protocol::CoreSaveInfoMap;
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

            // We use clone here because ideally we want snaps of our save files also to keep running this in parallel with futures saves
            save(save_info.clone());
        }
    }
}
/// A simple function that save in bincode files the adjusted resources CoreSaveInfoMap. Should occur everytime we modify that core resource in code,
/// Example: User modifies current skin, save!
fn save(save_info: CoreSaveInfoMap) {
    info!("Saving new information!");
    // Unwraps here because I dont see how one would be able to just change a const or a already initialized struct field type
    let mut f = BufWriter::new(File::create(SAVE_FILE_PATH).unwrap());
    let save_info_map = save_info;
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

/// This guy is gonna receive the sent save messages from client, and check if such action can be or not executed
/// If not, he is not gonna update confirmed after that client validates_predicted_confirmed should act accordingly to the mechanic
fn check_client_sent_core_information(
    mut save_from_client: EventReader<MessageEvent<SaveMessage>>,
    mut core_info_map: ResMut<CoreSaveInfoMap>,
    player_map: Res<ServerClientIdPlayerMap>,
    mut player_visual: Query<&mut PlayerVisuals>,
    mut connection_manager: ResMut<ServerConnectionManager>,
) {
    for save_message in save_from_client.read() {
        let mut message = save_message.message().clone();
        let client_id = message.id;
        // First -> Validate optional fields in save_message
        // Second -> Override coreinformation values according to new valuies
        // Third -> Send message to other clients
        if let Some(previous_core) = core_info_map.map.get_mut(&client_id) {
            let mut new_core_information = previous_core.clone(); // Start with previous core information

            // Validation stage for visual changes - Skip here if he doesnt pass validation
            if let Some(change_visual) = &message.change_char {
                // Perform any validation logic here
                info!("Validation for visual change: {:?}", change_visual);
                if let Some(player_entity) = player_map.map.get(&client_id) {
                    // Mutating confirmed entity in client to follow new old part
                    let mut server_visual = player_visual.get_mut(*player_entity).unwrap();
                    let body_part = &change_visual.body_part;
                    let old_part = server_visual.get_visual_mut(body_part);
                    let new_part = change_visual.path_to_part.clone();
                    *old_part = new_part;
                    new_core_information.player_visuals = server_visual.clone();
                }
            }

            // After validation stage change core information on map and save
            *previous_core = new_core_information;
            // Save updated core info map
            save(core_info_map.clone());
            // Broadcast save message to clients to act upon
            if connection_manager
                .send_message_to_target::<CommonChannel, SaveMessage>(
                    &mut message,
                    NetworkTarget::All,
                )
                .is_err()
            {
                warn!("Even tho server gave the okay couldnt broadcast message to all clients!")
            }
        }
    }
}
