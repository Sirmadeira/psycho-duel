use bevy::prelude::*;
use bevy::utils::Duration;
use egui::SharedEgui;
use lightyear::prelude::*;
use lightyear::shared::config::Mode;
use protocol::ProtocolPlugin;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

/// Shared configuration - Since this needs to be equal both in server and client, we shant leave it in core shared.
pub fn shared_config() -> SharedConfig {
    SharedConfig {
        // send an update every 100ms
        server_replication_send_interval: SERVER_REPLICATION_INTERVAL,
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode: Mode::Separate,
    }
}

/// Systems and protocols, that need to be shared between server and client will be stationed here.
pub struct CoreSharedPlugin;

// All mods in shared need to be pubbed
pub mod egui;
pub mod protocol;

impl Plugin for CoreSharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedEgui);
        app.add_plugins(ProtocolPlugin);
    }
}
