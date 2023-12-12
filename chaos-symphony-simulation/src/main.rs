#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

mod network;

use bevy::prelude::*;
use network::{NetworkBridge, NetworkPlugin};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(NetworkPlugin)
        .add_systems(Startup, connect);

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn connect(mut commands: Commands, bridge: Res<NetworkBridge>) {
    let connection = bridge.connection();
    connection.connect().expect("unable to connect");
    commands.spawn(connection);
}
