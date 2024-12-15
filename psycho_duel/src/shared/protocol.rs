use crate::client::egui::ChangeCharEvent;
use crate::client::egui::Parts;
use crate::shared::ClientId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use lightyear::prelude::client::ComponentSyncMode;
use lightyear::prelude::*;
use serde::Deserialize;
use serde::Serialize;
/// Essential struct that marks our player predicted entity.
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerMarker;

/// Points out who is the client id of the given player entity.
/// IMPORTANT- Every entity that is reasonably used, should have similar identifiers
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerId {
    pub id: ClientId,
}

/// Essential component utilized to tell me what exactly are the scenes player should have as children
/// Important- Why servertoclient? Well we wanna validate if everything occured succesfully, if so we are going to change in server via messages
/// Also to mantain a monolithic player entity
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerVisuals {
    /// Base skeleton to fit into
    pub skeleton: String,
    /// Character available head
    pub head: String,
    /// Character available torso
    pub torso: String,
    /// Character available legs
    pub leg: String,
    /// Character available weapon
    pub weapon_1: String,
}

impl Default for PlayerVisuals {
    fn default() -> Self {
        Self {
            head: String::from("characters/parts/suit_head.glb"),
            torso: String::from("characters/parts/scifi_torso.glb"),
            leg: String::from("characters/parts/witch_legs.glb"),
            weapon_1: String::from("weapons/katana.glb"),
            skeleton: String::from("characters/parts/main_skeleton.glb"),
        }
    }
}

impl PlayerVisuals {
    // Returns an iterator over the visual components. Inclu
    pub fn iter_visuals(&self) -> impl Iterator<Item = &String> {
        vec![&self.head, &self.torso, &self.leg, &self.skeleton].into_iter()
    }
    /// Returns a reference to the visual component corresponding to the given`Parts` enum
    /// Avoids the usage of uncessary match statements
    pub fn get_visual(&self, part: &Parts) -> &String {
        match part {
            Parts::Head => &self.head,
            Parts::Torso => &self.torso,
            Parts::Leg => &self.leg,
        }
    }
    pub fn get_visual_mut(&mut self, part: &Parts) -> &mut String {
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
    pub player_visuals: PlayerVisuals,
    pub player_id: PlayerId,
}

impl CoreInformation {
    /// Pass a client id - Get a semi defaulted core
    pub fn new(client_id: ClientId) -> Self {
        Self {
            player_id: PlayerId { id: client_id },
            player_visuals: PlayerVisuals::default(),
        }
    }
    // Pass a client id + current player visual, get a totally new core information
    pub fn total_new(client_id: ClientId, player_visuals: PlayerVisuals) -> Self {
        Self {
            player_id: PlayerId { id: client_id },
            player_visuals: player_visuals,
        }
    }
}

/// A bidirectional message utilized, to save things on server.
/// If one of the optional fields are passed we should validate the information on that event
#[derive(Event, Serialize, Deserialize, Clone, PartialEq)]
pub struct SaveMessage {
    pub save_info: CoreInformation,
    pub change_char: Option<ChangeCharEvent>,
}

/// Centralization plugin - Defines how our component will be synced (from server to client or client to server or bidirectional)
/// Defines what essential components need to be replicated among the two.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Essential infos - Channeldirection = Who sends message to who, is it  server to client, client to server?
        // Add prediction - Imagine  it like this - Either server or client, spawns entity with these shared components. Example - commands.spawn(PlayerMarker)
        // -> Via register_component and channel direction we define which should be transmissible. Important - the replicate component says - What entities should be replicated, or predicted, or interpolated and to whom
        // -> If when you added Replicated, you added a sync_targer prediction. You are going to spawn a predicted entity, if you added add_prediction in register_component
        // -> The predicted entity shall have that component
        // -> Componentsyncmode - Tells exactly, how many times we should replicate it to client or server.
        app.register_component::<PlayerMarker>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PlayerVisuals>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Name>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        // Replicated resources -  The workflow for replicated resources is as follows
        //-> First - Register in shared, as he is supposed to exist both in client and server
        //-> Second - Initialize him in server or vice versa depending on your channel direction
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
