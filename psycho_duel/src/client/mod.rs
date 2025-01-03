use crate::client::animation::ClientAnimationPlugin;
use crate::client::load_assets::LoadAssetsPlugin;
use crate::server::SERVER_ADDR;
use crate::shared::*;
use bevy::{prelude::*, window::ClosingWindow};
use camera::ClientCameraPlugin;
use egui::ClientEguiPlugin;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use player::ClientPlayerPlugin;
use protocol::CoreSaveInfoMap;
use skybox::SkyboxPlugin;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use world::ClientWorldPlugin;

/// Centralization plugin - When we pass in the cli the arg "client" this guy runs
pub struct CoreClientPlugin {
    /// This is one of the only few plugins that actually require an argument
    /// In this case we need t ograb
    pub client_id: u64,
}

/// Essential state for functionality - Basically tell me what is the current state of our app
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect)]
pub enum ClientAppState {
    #[default]
    /// Started loading assets - Once finish we can move forward
    LoadingAssets,
    // In game state - First stage of our game
    Game,
}

/// Although there is lightyear ClientConnection a very similar resource, he is not reflectable.
/// This one is and also he follows the we have control guideline although he is one of the few resource, not initialized immediately.
/// As he needs to be connected to have a client_id
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CoreEasyClient {
    client_id: ClientId,
}

pub mod camera;
// This guy is public because we need to share the Parts struct with the impl on shared
mod animation;
pub mod egui;
mod load_assets;
mod player;
mod skybox;
mod world;

impl Plugin for CoreClientPlugin {
    fn build(&self, app: &mut App) {
        // Client usually requires every single plugin available in bevy
        app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        }));

        // This looks weird but just imagine you are building a lot of plugins at once
        app.add_plugins(build_client_plugin(&self.client_id));

        // Add our shared plugin containing the protocol + other shared behaviour
        app.add_plugins(CoreSharedPlugin);

        //Add selfmade plugins
        app.add_plugins(ClientCameraPlugin);
        app.add_plugins(ClientEguiPlugin);
        app.add_plugins(ClientPlayerPlugin);
        app.add_plugins(ClientWorldPlugin);
        app.add_plugins(LoadAssetsPlugin);
        app.add_plugins(SkyboxPlugin);
        app.add_plugins(ClientAnimationPlugin);

        // Initializing center state of client
        app.init_state::<ClientAppState>();

        // Initialiazing core resources and replicated ones
        app.init_resource::<CoreSaveInfoMap>();

        // Add our client-specific logic. Here we will just connect to the server only when we have our assets loaded
        app.add_systems(OnEnter(ClientAppState::Game), connect_client);

        // Essential systems - Run in update because as reconnects may occur client id may vary, only prod tho.
        app.add_systems(Update, form_easy_client);

        // Observer checks if our client closed it is main window if so
        app.add_observer(on_app_exit_disconnect);

        // Debug
        app.register_type::<CoreEasyClient>();
    }
}

/// Here we create the lightyear [`ClientPlugins`], a series of plugins responsible to setup our base client.
fn build_client_plugin(client_id: &u64) -> ClientPlugins {
    // This is super temporary, we use this just to avoid overlapping addresses with other clients
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
        config: NetcodeConfig {
            // Ensures that if we dont listend from a client in 3 seconds server kills his replication entity
            client_timeout_secs: 3,
            ..default()
        },
    };
    let config = ClientConfig {
        // part of the config needs to be shared between the client and server
        shared: shared_config(),
        net: net_config,
        prediction: PredictionConfig {
            minimum_input_delay_ticks: 3,
            correction_ticks_factor: 2.5,
            ..default()
        },
        ..default()
    };
    ClientPlugins::new(config)
}

/// Connect to the server
fn connect_client(mut commands: Commands) {
    commands.connect_client();
}

///  When our app is closed we send a disconnect even to server
fn on_app_exit_disconnect(
    trigger: Trigger<OnAdd, ClosingWindow>,
    easy_client: Res<CoreEasyClient>,
    mut commands: Commands,
) {
    info!(
        "Client it {} closed it is main window {} sending disconnect event",
        easy_client.client_id,
        trigger.entity()
    );
    commands.disconnect_client();
}

/// Forms one of the most essential resources for us a resource, that stores our client_id.
/// Worh mentioning if for some unknow reason client id of this guy changes this guy will stand corrected
fn form_easy_client(mut connect_event: EventReader<ConnectEvent>, mut commands: Commands) {
    for event in connect_event.read() {
        let client_id = event.client_id();
        commands.insert_resource(CoreEasyClient {
            client_id: client_id,
        })
    }
}
