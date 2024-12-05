use crate::server::SERVER_ADDR;
use crate::shared::*;
use bevy::prelude::*;
pub use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

/// Here we create the lightyear [`ClientPlugins`], a series of plugins responsible to setup our base client.
fn build_client_plugin(client_id: &u64) -> ClientPlugins {
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, *client_id as u8)), 4000);

    // The NetConfig specifies how we establish a connection with the server.
    let net_config = NetConfig::Netcode {
        // Authentication is where you specify how the client should connect to the server
        // This is where you provide the server address
        auth: Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id: *client_id,
            private_key: Key::default(),
            protocol_id: 0,
        },
        // The IoConfig will specify the transport to use.
        io: IoConfig {
            // the address specified here is the client_address, because we open a UDP socket on the client
            transport: ClientTransport::UdpSocket(client_addr),
            ..default()
        },
        // We can use either Steam (in which case we will use steam sockets and there is no need to specify
        // our own io) or Netcode (in which case we need to specify our own io).
        config: NetcodeConfig::default(),
    };
    let config = ClientConfig {
        // part of the config needs to be shared between the client and server
        shared: shared_config(),
        net: net_config,
        ..default()
    };
    ClientPlugins::new(config)
}

/// Centralization plugin - When we pass in the cli the arg "client" this guy runs
pub struct CoreClientPlugin {
    /// This is one of the only few plugins that actually require an argument
    /// In this case we need t ograb
    pub client_id: u64,
}

impl Plugin for CoreClientPlugin {
    fn build(&self, app: &mut App) {
        // Client usually requires every single plugin available in bevy
        app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        }));

        // This looks weird but just imagine you are passing a series of args for a series of systems
        app.add_plugins(build_client_plugin(&self.client_id));

        // Add our shared plugin containing the protocol + other shared behaviour
        app.add_plugins(CoreSharedPlugin);

        // Add our client-specific logic. Here we will just connect to the server
        app.add_systems(Startup, connect_client);

        // Spawns our camera
        app.add_systems(Startup, spawn_camera);
    }
}

/// Connect to the server
fn connect_client(mut commands: Commands) {
    commands.connect_client();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle::default())
        .insert(MarkerPrimaryCamera)
        .insert(Name::new("MainCamera"));
}
