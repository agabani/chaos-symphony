use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::TryRecvError,
        Arc, Mutex,
    },
};

use bevy::{prelude::*, utils::tracing::instrument};
use chaos_symphony_network::{Connection, Payload, Server};

/// Network Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let (from_tokio, to_bevy) = std::sync::mpsc::channel();

        tokio::spawn(NetworkServer::bridge(from_tokio));

        app.insert_resource(NetworkServer {
            recv: Mutex::new(to_bevy),
        });
    }
}

/// Network Server.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct NetworkServer {
    recv: Mutex<std::sync::mpsc::Receiver<NetworkEndpoint>>,
}

impl NetworkServer {
    /// Bridges bevy and tokio runtime using channels.
    #[instrument(skip(sender))]
    async fn bridge(sender: std::sync::mpsc::Sender<NetworkEndpoint>) {
        let server = Server::new().unwrap();
        debug!("listening");

        loop {
            let Some(connecting) = server.accept().await else {
                panic!("Server network connection closed.");
            };

            let sender = sender.clone();

            tokio::spawn(async move {
                let span = error_span!("bridge", remote_address =% connecting.remote_address());
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

    /// Try to accept a new [`NetworkEndpoint`].
    pub fn try_accept(&self) -> Result<NetworkEndpoint, TryRecvError> {
        self.recv.lock().expect("poisoned").try_recv()
    }
}

/// Network Endpoint.
#[allow(clippy::module_name_repetitions)]
#[derive(Component)]
pub struct NetworkEndpoint {
    id: usize,
    is_disconnected: std::sync::atomic::AtomicBool,
    recv: Arc<Mutex<std::sync::mpsc::Receiver<NetworkRecv>>>,
    remote_address: std::net::SocketAddr,
    _send: tokio::sync::mpsc::UnboundedSender<NetworkSend>,
}

impl NetworkEndpoint {
    /// Creates a new [`NetworkEndpoint`].
    fn new(
        connection: &Connection,
        send: tokio::sync::mpsc::UnboundedSender<NetworkSend>,
        recv: std::sync::mpsc::Receiver<NetworkRecv>,
    ) -> Self {
        Self {
            id: connection.id(),
            is_disconnected: AtomicBool::new(false),
            recv: Arc::new(Mutex::new(recv)),
            remote_address: connection.remote_address(),
            _send: send,
        }
    }

    /// Bridges bevy and tokio runtime using channels.
    #[instrument(
        skip(connection, sender, _receiver),
        fields(
            id = connection.id(),
            remote_address =% connection.remote_address()
        )
    )]
    async fn bridge(
        connection: Connection,
        sender: std::sync::mpsc::Sender<NetworkRecv>,
        _receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkSend>,
    ) {
        debug!("connected");

        while let Ok(buf) = connection.recv().await {
            sender.send(NetworkRecv::Event(buf)).unwrap();
        }

        debug!("disconnected");
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

    /// Try receive.
    pub fn try_recv(&self) -> Result<NetworkRecv, TryRecvError> {
        let result = self.recv.lock().expect("poisoned").try_recv();

        if let Err(TryRecvError::Disconnected) = result {
            self.is_disconnected.store(true, Ordering::Relaxed);
        }

        result
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
pub enum NetworkSend {}
