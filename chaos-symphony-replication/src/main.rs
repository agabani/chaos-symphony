#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

use bevy::prelude::*;
use chaos_symphony_network::Server;

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        let server = Server::new().unwrap();
        println!("[network] listening");

        loop {
            let Some(connecting) = server.accept().await else {
                panic!("Server network connection closed.");
            };
            println!("[network] connecting");

            tokio::spawn(async move {
                let connection = connecting.accept().await.unwrap();
                println!("[network] connected");

                while let Ok(buf) = connection.recv().await {
                    println!("[network] recv: {buf:?}");
                }

                println!("[network] disconnected");
            });
        }
    });

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.run();
}
