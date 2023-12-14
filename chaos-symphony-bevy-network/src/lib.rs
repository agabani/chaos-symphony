#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Bevy Network

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::TryRecvError,
        Arc, Mutex,
    },
    time::Duration,
};

use bevy::{
    prelude::*,
    utils::{tracing::instrument, Uuid},
};
use chaos_symphony_network::{Client, Connection, Payload, Server};

/// Network Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkPlugin {
    /// Client.
    pub client: bool,

    /// Server.
    pub server: bool,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        if self.client {
            let (from_bevy, to_tokio) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(NetworkClient::bridge(to_tokio));
            app.insert_resource(NetworkClient::new(from_bevy));
        }

        if self.server {
            let (from_tokio, to_bevy) = std::sync::mpsc::channel();
            tokio::spawn(NetworkServer::bridge(from_tokio));
            app.insert_resource(NetworkServer::new(to_bevy));
        }

        app.insert_resource(KeepAliveTimer::new())
            .add_systems(Update, keep_alive);
    }
}

/// Network Client.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct NetworkClient {
    sender: tokio::sync::mpsc::UnboundedSender<std::sync::mpsc::Sender<NetworkEndpoint>>,
}

impl NetworkClient {
    /// Creates a new [`NetworkClient`].
    fn new(
        sender: tokio::sync::mpsc::UnboundedSender<std::sync::mpsc::Sender<NetworkEndpoint>>,
    ) -> Self {
        Self { sender }
    }

    /// Connect.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn connect(
        &self,
    ) -> Result<
        Connecting,
        tokio::sync::mpsc::error::SendError<std::sync::mpsc::Sender<NetworkEndpoint>>,
    > {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.sender.send(sender).map(|()| Connecting {
            receiver: Mutex::new(receiver),
        })
    }

    /// Bridges bevy-tokio runtime using channels.
    #[instrument(name = "network_client", skip(receiver))]
    async fn bridge(
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<
            std::sync::mpsc::Sender<NetworkEndpoint>,
        >,
    ) {
        let client = Client::new().expect("unable to bind to port or find certificate");
        debug!("started");

        loop {
            let Some(sender) = receiver.recv().await else {
                panic!("connection closed");
            };

            let connecting = client.connect().unwrap();

            tokio::spawn(async move {
                let span = error_span!(
                    "network_client", remote_address =% connecting.remote_address()
                );
                let _guard = span.enter();
                debug!("connecting");

                let connection = connecting.accept().await.unwrap();

                let (from_bevy, to_tokio) = tokio::sync::mpsc::unbounded_channel();
                let (from_tokio, to_bevy) = std::sync::mpsc::channel();

                sender
                    .send(NetworkEndpoint::new(&connection, from_bevy, to_bevy))
                    .unwrap();

                NetworkEndpoint::bridge(connection, from_tokio, to_tokio).await;
            });
        }
    }
}

/// Network Endpoint.
#[allow(clippy::module_name_repetitions)]
#[derive(Component)]
pub struct NetworkEndpoint {
    id: usize,
    is_disconnected: std::sync::atomic::AtomicBool,
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<NetworkRecv>>>,
    remote_address: std::net::SocketAddr,
    sender: tokio::sync::mpsc::UnboundedSender<NetworkSend>,
}

impl NetworkEndpoint {
    /// Creates a new [`NetworkEndpoint`].
    #[must_use]
    pub fn new(
        connection: &Connection,
        sender: tokio::sync::mpsc::UnboundedSender<NetworkSend>,
        receiver: std::sync::mpsc::Receiver<NetworkRecv>,
    ) -> Self {
        Self {
            id: connection.id(),
            is_disconnected: AtomicBool::new(false),
            receiver: Arc::new(Mutex::new(receiver)),
            remote_address: connection.remote_address(),
            sender,
        }
    }

    /// Returns the id of this [`NetworkEndpoint`].
    pub fn id(&self) -> usize {
        self.id
    }

    /// Is disconnected.
    pub fn is_disconnected(&self) -> bool {
        self.is_disconnected.load(Ordering::Relaxed)
    }

    /// Returns the remote address of this [`NetworkEndpoint`].
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }

    /// Try receive payload.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected or empty.
    ///
    /// # Panics
    ///
    /// Will panic if [`Mutex`] is poisoned.
    pub fn try_recv(&self) -> Result<NetworkRecv, TryRecvError> {
        let result = self.receiver.lock().expect("poisoned").try_recv();

        if let Err(TryRecvError::Disconnected) = result {
            self.is_disconnected.store(true, Ordering::Relaxed);
        }

        result
    }

    /// Try send payload.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(
        &self,
        payload: Payload,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<NetworkSend>> {
        let result = self.sender.send(NetworkSend::Event(payload));

        if result.is_err() {
            self.is_disconnected.store(true, Ordering::Relaxed);
        }

        result
    }

    /// Bridges bevy-tokio runtime using channels.
    #[instrument(
        name = "network_endpoint",
        skip(connection, sender, receiver),
        fields(
            id = connection.id(),
            remote_address =% connection.remote_address()
        )
    )]
    async fn bridge(
        connection: Connection,
        sender: std::sync::mpsc::Sender<NetworkRecv>,
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkSend>,
    ) {
        debug!("connected");

        loop {
            tokio::select! {
                result = connection.recv() => {
                    // inbound traffic
                    match result {
                        Ok(payload) =>  {
                            if sender.send(NetworkRecv::Event(payload)).is_err() {
                                break;
                            }
                        },
                        Err(_) => {
                            break;
                        },
                    };
                }
                result = receiver.recv() => {
                    // outbound traffic
                    match result {
                        Some(instruction) => {
                            match instruction {
                                NetworkSend::Event(payload) => {
                                    if connection.send(payload).await.is_err() {
                                        break;
                                    }
                                },
                            }
                        },
                        None => {
                            break;
                        },
                    };
                }
            }
        }

        debug!("disconnected");
    }
}

/// Network Recv.
#[allow(clippy::module_name_repetitions)]
pub enum NetworkRecv {
    /// Event.
    Event(Payload),
}

/// Network Send.
#[allow(clippy::module_name_repetitions)]
pub enum NetworkSend {
    /// Event.
    Event(Payload),
}

/// Network Server.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct NetworkServer {
    receiver: Mutex<std::sync::mpsc::Receiver<NetworkEndpoint>>,
}

impl NetworkServer {
    /// Creates a new [`NetworkServer`].
    fn new(receiver: std::sync::mpsc::Receiver<NetworkEndpoint>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
        }
    }

    /// Try to receive a new [`NetworkEndpoint`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected or empty.
    ///
    /// # Panics
    ///
    /// Will panic if [`Mutex`] is poisoned.
    pub fn try_recv(&self) -> Result<NetworkEndpoint, TryRecvError> {
        self.receiver.lock().expect("poisoned").try_recv()
    }

    /// Bridges bevy-tokio runtime using channels.
    #[instrument(name = "network_server", skip(sender))]
    async fn bridge(sender: std::sync::mpsc::Sender<NetworkEndpoint>) {
        let server = Server::new().expect("unable to bind to port or find certificate");
        debug!("started");

        loop {
            let Some(connecting) = server.accept().await else {
                panic!("connection closed");
            };

            let sender = sender.clone();

            tokio::spawn(async move {
                let span = error_span!(
                    "network_server", remote_address =% connecting.remote_address()
                );
                let _guard = span.enter();
                debug!("connecting");

                let connection = connecting.accept().await.unwrap();

                let (from_bevy, to_tokio) = tokio::sync::mpsc::unbounded_channel();
                let (from_tokio, to_bevy) = std::sync::mpsc::channel();

                sender
                    .send(NetworkEndpoint::new(&connection, from_bevy, to_bevy))
                    .unwrap();

                NetworkEndpoint::bridge(connection, from_tokio, to_tokio).await;
            });
        }
    }
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
fn keep_alive(
    time: Res<Time>,
    mut timer: ResMut<KeepAliveTimer>,
    query: Query<(Entity, &NetworkEndpoint)>,
) {
    if timer.inner.tick(time.delta()).just_finished() {
        query.for_each(|(entity, endpoint)| {
            let payload = Payload {
                id: Uuid::new_v4().to_string(),
                endpoint: "/ping".to_string(),
                properties: HashMap::new(),
            };

            if endpoint.try_send(payload).is_err() {
                let span = warn_span!(
                    "keep_alive",
                    entity =? entity,
                    id = endpoint.id(),
                    remote_address =% endpoint.remote_address()
                );
                let _guard = span.enter();

                warn!("unable to send ping");
            };
        });
    }
}

/// Connecting.
#[derive(Component)]
pub struct Connecting {
    receiver: Mutex<std::sync::mpsc::Receiver<NetworkEndpoint>>,
}

impl Connecting {
    /// Try Poll.
    ///
    /// Will disconnect bevy-tokio bridge on first [`Poll::Ready`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected or empty.
    ///
    /// # Panics
    ///
    /// Will panic if [`Mutex`] is poisoned.
    pub fn try_poll(&self) -> Poll<Result<NetworkEndpoint, PollError>> {
        match self.receiver.lock().expect("poisoned").try_recv() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(TryRecvError::Disconnected) => Poll::Ready(Err(PollError::Disconnected)),
            Err(TryRecvError::Empty) => Poll::Pending,
        }
    }
}

/// Poll.
pub enum Poll<T> {
    /// Ready.
    Ready(T),

    /// Pending
    Pending,
}

/// Poll Error.
#[derive(Debug)]
pub enum PollError {
    /// Disconnected.
    Disconnected,
}
