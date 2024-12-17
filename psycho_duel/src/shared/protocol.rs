use crate::client::egui::ChangeCharEvent;
use crate::client::egui::Parts;
use crate::shared::ClientId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use lightyear::prelude::client::ComponentSyncMode;
use lightyear::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Essential struct that marks our player predicted entity.
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerMarker;

/// Points out who is the client id of the given player entity.
/// IMPORTANT- Every entity that is reasonably used, should have similar identifiers
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerId {
    /// Base client id of this player
    pub id: ClientId,
}

/// Component that tell me exactly what items that player has available to him
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Inventory {
    items: HashMap<String, Item>,
}

/// Item is an abstraction utilized to easy our management of what player has, when it comes to Assets
/// Things like, guns, visuals, should be items
#[derive(Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Item {
    /// Name of that item - Mostly used for pretty egui
    pub name: Name,
    /// File path - File path to that grab that item via AssetCollections
    pub file_path: String,
}

impl Item {
    /// Creates an item from it is given file path, grabing the last part from slice.
    pub fn new_from_filepath(file_path: &str) -> Self {
        let name_str = file_path
            .split("/")
            .last()
            .unwrap_or(&file_path)
            .to_string();
        Self {
            name: Name::new(name_str),
            file_path: file_path.to_string(),
        }
    }
}
/// Display trait for item, shows us his name
impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Essential component utilized to tell me what exactly are the scenes player should have as children
/// Important- Why servertoclient? Well we wanna validate if everything occured succesfully, if so we are going to change in server via messages
/// Also to mantain a monolithic player entity
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerVisuals {
    /// Base skeleton to fit into
    pub skeleton: Item,
    /// Character available head
    pub head: Item,
    /// Character available torso
    pub torso: Item,
    /// Character available legs
    pub leg: Item,
    /// Character available weapon
    pub weapon_1: Item,
}

impl Default for PlayerVisuals {
    fn default() -> Self {
        Self {
            head: Item::new_from_filepath("characters/parts/suit_head.glb"),
            torso: Item::new_from_filepath("characters/parts/scifi_torso.glb"),
            leg: Item::new_from_filepath("characters/parts/witch_legs.glb"),
            weapon_1: Item::new_from_filepath("weapons/katana.glb"),
            skeleton: Item::new_from_filepath("characters/parts/main_skeleton.glb"),
        }
    }
}

impl PlayerVisuals {
    /// Returns an iterator over the visual components. Good iterator for when spawning first the entity
    pub fn iter_visuals(&self) -> impl Iterator<Item = &Item> {
        vec![&self.head, &self.torso, &self.leg, &self.skeleton].into_iter()
    }
    /// Returns a reference to the visual component corresponding to the given`Parts` enum
    /// Avoids the usage of uncessary match statements
    pub fn get_visual(&self, part: &Parts) -> &Item {
        match part {
            Parts::Head => &self.head,
            Parts::Torso => &self.torso,
            Parts::Leg => &self.leg,
        }
    }
    /// Returns a mutable reference to the visual component corresponding to the given`Parts` enum
    /// Avoids the usage of uncessary match statements
    pub fn get_visual_mut(&mut self, part: &Parts) -> &mut Item {
        match part {
            Parts::Head => &mut self.head,
            Parts::Torso => &mut self.torso,
            Parts::Leg => &mut self.leg,
        }
    }
}

/// Our save resource map, it is gonna store all types of core information really important for our mechanics.
/// Initially this is monolithic, meaning we only have one save info map that stores basically all of user info later we can make subdivisions and subfiles
#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Default)]
#[reflect(Resource)]
pub struct CoreSaveInfoMap {
    pub map: HashMap<ClientId, CoreInformation>,
}

/// A centralization struct - That shall store everything that NEEDs to be saved about that specific client
/// It also stores a client_id pointer
#[derive(Bundle, Serialize, Deserialize, Reflect, Clone, Debug, PartialEq)]
pub struct CoreInformation {
    pub player_id: PlayerId,
    pub player_visuals: PlayerVisuals,
    pub currency: Currency,
}

impl CoreInformation {
    /// Pass a client id - Get a semi defaulted core
    pub fn new(client_id: ClientId) -> Self {
        Self {
            player_id: PlayerId { id: client_id },
            player_visuals: PlayerVisuals::default(),
            currency: Currency::default(),
        }
    }
}

/// A bidirectional message utilized, to save things on server.
/// If one of the optional fields are passed we should validate the information.
#[derive(Event, Serialize, Deserialize, Clone, PartialEq)]
pub struct SaveMessage {
    pub id: ClientId,
    pub change_char: Option<ChangeCharEvent>,
    pub change_currency: Option<Currency>,
}

/// Component responsible to tell me how much money a specific client id has
/// It has mathematical function to ease our usage
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Currency {
    pub amount: f32,
}

impl Default for Currency {
    fn default() -> Self {
        Self { amount: 1.0 }
    }
}

impl Currency {
    pub fn add(&mut self, value: f32) {
        self.amount += value;
    }
    pub fn sub(&mut self, value: f32) {
        self.amount -= value;
    }
}

/// Centralization plugin - Defines how our component will be synced (from server to client or client to server or bidirectional)
/// Defines what essential components need to be replicated among the two.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // -> First - Spawn an entity with replicate component on server, after you do that this api applies it is logic
        // -> Second - Register component, means that component will be available on the replicated entity on client
        // -> Third - ChannelDirection tells me if it is server to client or client  to server, the replication direction.
        // Most cases is the first method (Server authoritative avoids hacks)
        // -> Fourth - Add prediction, inserts that component on the predicted entity.
        // -> Fifth - ComponentSyncMode tell me, how many time we should send that information from confirmed entity, to predicted.
        app.register_component::<PlayerMarker>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PlayerVisuals>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Inventory>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Currency>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Name>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Simple);

        // Replicated resources -  The workflow for replicated resources is as follows
        //-> First - Register in shared, as he is supposed to exist both in client and server
        //-> Second - Initialize him in server and client
        //-> Third - Do commands.replicate to start replicating him when necessary
        app.register_resource::<CoreSaveInfoMap>(ChannelDirection::ServerToClient);

        // Self-made messages - The workflow for messages is as follows:
        // -> First register message
        // -> Send her via clientconnectionmessager using send_message function with all of it is shenanigans
        // -> Read it via EventReader<MessageEvent<>>
        app.register_message::<SaveMessage>(ChannelDirection::Bidirectional);

        // Debug registering
        app.register_type::<PlayerId>();
        app.register_type::<PlayerVisuals>();
        app.register_type::<CoreSaveInfoMap>();
    }
}
