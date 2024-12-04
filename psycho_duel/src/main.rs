use bevy::prelude::*;
use clap::Parser;
use client::CoreClientPlugin;
use server::CoreServerPlugin;

mod client;
mod server;
mod shared;

/// This struct simbolizes our console logic interface. With clap we can easily
/// Grab the agument passed after cargo run, and make logic according to it
#[derive(Parser, PartialEq, Debug)]
pub enum Cli {
    /// The program will act as server
    Server,
    /// The program will act as a client
    Client,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    // Here we match the keyword passed by our cli and run the according plugin
    // Worth noting, since your game is competitive we only will run this in separate mode
    // Meaning we wont have host client, and server-client types.
    match cli {
        Cli::Server => app.add_plugins(CoreServerPlugin),
        Cli::Client => app.add_plugins(CoreClientPlugin),
    };

    app.run();
}
