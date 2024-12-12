use crate::shared::ClientId;
use bevy::prelude::*;
use lightyear::prelude::client::ComponentSyncMode;
use lightyear::prelude::AppComponentExt;
use lightyear::prelude::ChannelDirection;
use serde::Deserialize;
use serde::Serialize;

/// Essential struct that marks our player predicted entity.
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerMarker;

/// Points out who is the client id of the given player entity.
/// IMPORTANT- Every entity that is reasonably used, should have similar identifiers
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerId {
    pub id: ClientId,
}

/// Essential component utilized to tell me what exactly are
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
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
        app.register_component::<PlayerVisuals>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Name>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        // Debug registering
        app.register_type::<PlayerId>();
        app.register_type::<PlayerVisuals>();
    }
}
