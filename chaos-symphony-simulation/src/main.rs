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

        let recv = {
            let connection = connection.clone();
            tokio::spawn(async move {
                while let Ok(buf) = connection.recv().await {
                    println!("[network] recv: {buf:?}");
                }
            })
        };

        let send = {
            let connection = connection.clone();
            tokio::spawn(async move {
                loop {
                    connection
                        .send(chaos_symphony_network::Payload {
                            id: "00000000".to_string(),
                            endpoint: "/ping".to_string(),
                            properties: std::collections::HashMap::new(),
                        })
                        .await
                        .unwrap();

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            })
        };

        tokio::select! {
            _ = recv => {},
            _ = send => {},
        };

        println!("[network] disconnected");
    });

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.run();
}
