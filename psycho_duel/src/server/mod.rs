use crate::shared::*;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub(crate) const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

/// Here we create the lightyear [`ServerPlugins`], a series of system responsible for setuping the logic of our server
/// It is replication interval, if he shall have input delay, and other similar aspects.
fn build_server_plugin() -> ServerPlugins {
    // The NetConfig specifies how we establish a connection with the server.
    // We can use either Steam (in which case we will use steam sockets and there is no need to specify
    // our own io) or Netcode (in which case we need to specify our own io).
    let net_config = NetConfig::Netcode {
        // The IoConfig will specify the transport to use.
        io: IoConfig {
            // the address specified here is the server_address, because we open a UDP socket on the server
            transport: ServerTransport::UdpSocket(SERVER_ADDR),
            ..default()
        },
        config: NetcodeConfig::default(),
    };
    let config = ServerConfig {
        // part of the config needs to be shared between the client and server
        shared: shared_config(),
        // we can specify multiple net configs here, and the server will listen on all of them
        // at the same time. Here we will only use one
        net: vec![net_config],
        replication: ReplicationConfig {
            // we will send updates to the clients every 100ms
            send_interval: SERVER_REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };
    ServerPlugins::new(config)
}

/// Centralization plugin - When we pass in the cli the arg "server" this guy runs
pub struct CoreServerPlugin;

impl Plugin for CoreServerPlugin {
    fn build(&self, app: &mut App) {
        // Different from client server doesnt require a lot of things, we usually shouldnt have a screen or render anything on him.
        // But as we are in development stage we gonna live it the default ones
        app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        }));
        // Add lightyear plugins
        app.add_plugins(build_server_plugin());

        // Add our shared plugin containing the protocol + other shared behaviour
        app.add_plugins(CoreSharedPlugin);

        // Add our server-specific logic. Here we will just start listening for incoming connections
        app.add_systems(Startup, start_server);
    }
}

/// Start the server
fn start_server(mut commands: Commands) {
    commands.start_server();
}
