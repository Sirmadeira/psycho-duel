use bevy::prelude::*;
use bevy::utils::Duration;
use egui::SharedEgui;
use lightyear::prelude::*;
use lightyear::shared::config::Mode;
use protocol::ProtocolPlugin;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

/// Systems and protocols, that need to be shared between server and client will be stationed here.
/// Warning - Whenever adjusting shared plugins, also reset server. I tend to make that mistake
pub struct CoreSharedPlugin;

/// Reliable ordered channel utilized for our own self made messages
#[derive(Channel)]
pub struct CommonChannel;

// All mods in shared need to be pubbed
pub mod egui;
pub mod protocol;

impl Plugin for CoreSharedPlugin {
    fn build(&self, app: &mut App) {
        // Protocol plugin- SUPER DUPER IMPORTANT
        app.add_plugins(ProtocolPlugin);

        // Self made plugins
        app.add_plugins(SharedEgui);

        //Self made channels
        app.add_channel::<CommonChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}
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
