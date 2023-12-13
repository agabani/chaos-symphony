use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network::{Client, Connection, Payload};

/// Network Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let (from_bevy, to_tokio) = tokio::sync::mpsc::unbounded_channel();
        let (from_tokio, to_bevy) = std::sync::mpsc::channel();

        tokio::spawn(bridge(from_tokio, to_tokio));

        app.insert_resource(NetworkBridge {
            send: from_bevy,
            recv: Arc::new(Mutex::new(to_bevy)),
        })
        .insert_resource(KeepAliveTimer::new())
        .add_systems(Update, keep_alive);
    }
}

/// Bridges bevy and tokio runtime using channels.
async fn bridge(
    _sender: std::sync::mpsc::Sender<()>,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkSend>,
) {
    let client = Client::new().unwrap();
    let mut connection: Option<Connection> = None;

    while let Some(send) = receiver.recv().await {
        match send {
            NetworkSend::Connect => {
                println!("[network] connect");
                let connecting = client.connect().unwrap();
                println!("[network] connecting");
                connection = Some(connecting.accept().await.unwrap());
                println!("[network] connected");
            }
            NetworkSend::Event(payload) => {
                if let Some(connection) = &connection {
                    connection.send(payload).await.unwrap();
                }
            }
        }
    }

    panic!("[network] channel dropped");
}

/// Network Bridge.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct NetworkBridge {
    send: tokio::sync::mpsc::UnboundedSender<NetworkSend>,
    recv: Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
}

impl NetworkBridge {
    /// Creates a new [`NetworkEndpoint`].
    pub fn endpoint(&self) -> NetworkEndpoint {
        NetworkEndpoint {
            send: self.send.clone(),
            _recv: self.recv.clone(),
        }
    }
}

/// Network Endpoint.
#[allow(clippy::module_name_repetitions)]
#[derive(Component)]
pub struct NetworkEndpoint {
    send: tokio::sync::mpsc::UnboundedSender<NetworkSend>,
    _recv: Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
}

impl NetworkEndpoint {
    /// Connect to a server.
    pub fn connect(&self) -> Result<(), tokio::sync::mpsc::error::SendError<NetworkSend>> {
        self.send.send(NetworkSend::Connect)
    }

    /// Send payload.
    pub fn send(
        &self,
        payload: Payload,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<NetworkSend>> {
        self.send.send(NetworkSend::Event(payload))
    }
}

/// Network Send.
#[allow(clippy::module_name_repetitions)]
pub enum NetworkSend {
    /// Connect.
    Connect,

    /// Event.
    Event(Payload),
}

/// Keep Alive Timer.
#[derive(Resource)]
struct KeepAliveTimer {
    inner: Timer,
}

impl KeepAliveTimer {
    /// Creates a new [`KeepAliveTimer`].
    fn new() -> Self {
        Self {
            inner: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

/// Keeps connection alive by periodically sending pings.
#[allow(clippy::needless_pass_by_value)]
fn keep_alive(time: Res<Time>, mut timer: ResMut<KeepAliveTimer>, query: Query<&NetworkEndpoint>) {
    if timer.inner.tick(time.delta()).just_finished() {
        query.for_each(|connection| {
            connection
                .send(Payload {
                    id: Uuid::new_v4().to_string(),
                    endpoint: "/ping".to_string(),
                    properties: HashMap::new(),
                })
                .unwrap();
        });
    }
}
