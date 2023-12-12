#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

use bevy::prelude::*;
use chaos_symphony_network::Client;

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        let client = Client::new().unwrap();

        let connecting = client.connect().unwrap();
        println!("[network] connecting");

        let connection = connecting.accept().await.unwrap();
        println!("[network] connected");

        while let Ok(buf) = connection.recv().await {
            println!("[network] recv: {buf:?}");
        }

        println!("[network] disconnected");
    });

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.run();
}
