use crate::server::SERVER_ADDR;
use crate::shared::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
pub use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

#[derive(Component, Reflect)]
pub struct MarkerPrimaryCamera;

/// Here we create the lightyear [`ClientPlugins`], a series of plugins responsible to setup our base client.
fn build_client_plugin() -> ClientPlugins {
    // The NetConfig specifies how we establish a connection with the server.
    let net_config = NetConfig::Netcode {
        // Authentication is where you specify how the client should connect to the server
        // This is where you provide the server address
        auth: Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id: 0,
            private_key: Key::default(),
            protocol_id: 0,
        },
        // The IoConfig will specify the transport to use.
        io: IoConfig {
            // the address specified here is the client_address, because we open a UDP socket on the client
            transport: ClientTransport::UdpSocket(CLIENT_ADDR),
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
pub struct CoreClientPlugin;

impl Plugin for CoreClientPlugin {
    fn build(&self, app: &mut App) {
        // Client usually requires every single plugin available in bevy
        app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            ..default()
        }));

        // This looks weird but just imagine you are passing a series of args for a series of systems
        app.add_plugins(build_client_plugin());

        // Add our shared plugin containing the protocol + other shared behaviour
        app.add_plugins(CoreSharedPlugin);

        // Add our client-specific logic. Here we will just connect to the server
        app.add_systems(Startup, connect_client);

        // Spawns our camera
        app.add_systems(Startup, spawn_camera);

        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);

        app.add_systems(Update, inspector_ui);
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

/// This is really close to what happens when you utilize worldinspectorplugin from bevy-inspector-egui
/// The difference is this guy is configurable meaning we can adjust him according to resolution and such
fn inspector_ui(world: &mut World) {
    // This is basically comp_egui_context: Query<&EguiContext, With<PrimaryWindow>>,
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        info_once!("Manage to grab egui form world");
        // We clone here to avoid changin other eguis context
        let mut egui_context = egui_context.clone();

        //  Imagine this as nesting so first comes window, when we do,
        // add_content closure ui we are ensuring that scroll area is child of window.
        // All you need to do is add more and more .show to make heavier nests. And call ui a lot if you want to make buttons and such
        egui::Window::new("UI")
            // .min_size((100.0, 100.0))
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // This is equivalent to "world inspector plugin"
                    bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

                    egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                        bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                            world, ui,
                        );
                    });

                    // ui.heading("Entities");
                    // bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
                })
            });
    } else {
        warn!("Something is terribly wrong cant grab egui context");
        return;
    };
}
