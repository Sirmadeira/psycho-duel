use crate::server::protocol::CoreInformation;
use crate::shared::protocol::CoreSaveInfoMap;
use bevy::prelude::*;
use bincode::{deserialize_from, serialize_into};
use lightyear::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind};

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
    }
}

/// A simple function made to ensure that everyone knows to whon we are replicating this resource to
fn replicate_resource(mut commands: Commands) {
    commands.replicate_resource::<CoreSaveInfoMap, CommonChannel>(NetworkTarget::All);
}

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
            // Handle a new client by creating a default core information
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
    info!("Saving new file!");
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
