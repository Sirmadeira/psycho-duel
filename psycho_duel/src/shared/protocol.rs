use crate::shared::ClientId;
use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;
use lightyear::prelude::ChannelDirection;
use serde::Deserialize;
use serde::Serialize;

/// Points out who is the client id of the given player entity.
/// IMPORTANT- Every entity that is reasonably used, should have similar identifiers
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerId {
    pub id: ClientId,
}

/// Centralization plugin - Defines how our component will be synced (from server to client or client to server or bidirectional)
/// Defines what essential components need to be replicated among the two.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient);
        app.register_component::<Name>(ChannelDirection::Bidirectional);
    }
}
