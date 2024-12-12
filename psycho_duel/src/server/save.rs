use bevy::prelude::*;
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind};

/// Plugin utilized to store specific username info, for example: What visuals he currently have available? What itens he bought? The list goes on
/// We shall store that locally via bin files in our server, once he passes the validation phase (TBD). We will return that precious info via resource
pub struct SavePlugin;

/// A sample of what a save resource looks like.
/// Initially this is monolithic, meaning we only have one save info map that stores basically all of user info later we can make subdivisions and subfiles
#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Default)]
#[reflect(Resource)]
pub struct SaveInfoMap;

const SAVE_FILE_PATH: &str = "./psycho_duel/src/server/save_files/player_info.bar";

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_or_read_save_file);
    }
}

/// A simple function that save in bincode files the adjusted save_info_map. Should occur everytime we modify our internal resource in code,
/// Example: User modifies current skin, save!
fn save(save_info: &SaveInfoMap) {
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

            let save_info: SaveInfoMap = deserialize_from(buf_reader).expect(
                "If this breaks is because you changed SavedInfoMap fields just delete old save folder",
            );

            commands.insert_resource(save_info);
        }
        // If not okay because we didnt found the file. We create a new file and inititialize a default value for our save resource.
        Err(err) if err.kind() == ErrorKind::NotFound => {
            info!("File doesnt currently exist creating a default SaveInfoMap");
            let mut f = BufWriter::new(
                File::create(SAVE_FILE_PATH).expect("To be able to create new file"),
            );
            
            let save_info = SaveInfoMap::default();
            serialize_into(&mut f, &save_info)
                .expect("To be able to serialize into new file on startup");

            commands.insert_resource(save_info);
        }
        Err(err) => {
            panic!("Failed to open save file for an unexpected reason: {}", err);
        }
    }
}
