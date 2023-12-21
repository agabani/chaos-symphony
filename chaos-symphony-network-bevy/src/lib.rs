#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Network Bevy

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::TryRecvError,
        Arc,
    },
};

use bevy::{prelude::*, utils::tracing::instrument};
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network::{AcceptError, Client, Connection, Message, RecvError, Server};

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
    }
}

/// Network Client.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct NetworkClient {
    sender: tokio::sync::mpsc::UnboundedSender<
        std::sync::mpsc::Sender<Result<NetworkEndpoint, AcceptError>>,
    >,
}

impl NetworkClient {
    /// Creates a new [`NetworkClient`].
    fn new(
        sender: tokio::sync::mpsc::UnboundedSender<
            std::sync::mpsc::Sender<Result<NetworkEndpoint, AcceptError>>,
        >,
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
        tokio::sync::mpsc::error::SendError<
            std::sync::mpsc::Sender<Result<NetworkEndpoint, AcceptError>>,
        >,
    > {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.sender.send(sender).map(|()| Connecting {
            inner: Future::new(receiver),
        })
    }

    /// Bridges bevy-tokio runtime using channels.
    #[instrument(name = "network_client", skip(receiver))]
    async fn bridge(
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<
            std::sync::mpsc::Sender<Result<NetworkEndpoint, AcceptError>>,
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

                let connection = match connecting.accept().await {
                    Ok(connection) => connection,
                    Err(error) => {
                        sender.send(Err(error)).unwrap();
                        return;
                    }
                };

                let (from_bevy, to_tokio) = tokio::sync::mpsc::unbounded_channel();
                let (from_tokio, to_bevy) = std::sync::mpsc::channel();

                sender
                    .send(Ok(NetworkEndpoint::new(&connection, from_bevy, to_bevy)))
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
    receiver: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<NetworkRecv>>>,
    remote_address: SocketAddr,
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
            receiver: Arc::new(std::sync::Mutex::new(receiver)),
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

    /// Try receive message.
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

    /// Try send blocking.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send_blocking(
        &self,
        message: Message,
    ) -> Result<Future<Message>, tokio::sync::mpsc::error::SendError<NetworkSend>> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let result = self.sender.send(NetworkSend::Blocking { message, sender });

        if result.is_err() {
            self.is_disconnected.store(true, Ordering::Relaxed);
        }

        result.map(|()| Future::new(receiver))
    }

    /// Try send non blocking.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send_non_blocking(
        &self,
        message: Message,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<NetworkSend>> {
        let result = self.sender.send(NetworkSend::NonBlocking { message });

        if result.is_err() {
            self.is_disconnected.store(true, Ordering::Relaxed);
        }

        result
    }

    /// Bridges bevy-tokio runtime using channels.
    #[instrument(
        name = "network_endpoint_bridge",
        skip(connection, sender, receiver),
        fields(
            id = connection.id(),
            remote_address =% connection.remote_address()
        )
    )]
    async fn bridge(
        connection: Connection,
        sender: std::sync::mpsc::Sender<NetworkRecv>,
        receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkSend>,
    ) {
        debug!("connected");

        let database = Arc::new(tokio::sync::Mutex::new(HashMap::<
            String,
            std::sync::mpsc::Sender<Message>,
        >::new()));

        let (error_tx, mut error_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        let quit_inbound = Self::bridge_inbounds(
            error_tx.clone(),
            database.clone(),
            connection.clone(),
            sender,
        );
        let quit_outbound = Self::bridge_outbounds(error_tx, database, connection, receiver);

        error_rx.recv().await;

        drop(quit_inbound);
        drop(quit_outbound);

        debug!("disconnected");
    }

    /// Bridges inbounds bevy-tokio runtime using channels.
    #[instrument(
        name = "network_endpoint_bridge_inbounds",
        skip(error_tx, database, connection, sender),
        fields(
            id = connection.id(),
            remote_address =% connection.remote_address()
        )
    )]
    fn bridge_inbounds(
        error_tx: tokio::sync::mpsc::UnboundedSender<()>,
        database: Arc<tokio::sync::Mutex<HashMap<String, std::sync::mpsc::Sender<Message>>>>,
        connection: Connection,
        sender: std::sync::mpsc::Sender<NetworkRecv>,
    ) -> tokio::sync::mpsc::Sender<()> {
        let (quit_tx, mut quit_rx) = tokio::sync::mpsc::channel::<()>(1);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = connection.recv() => {
                        tokio::spawn(Self::bridge_inbound(error_tx.clone(), database.clone(), sender.clone(), result));
                    }
                    _ = quit_rx.recv() => {
                        debug!("quit received");
                        return;
                    }
                }
            }
        });
        quit_tx
    }

    /// Bridges outbounds bevy-tokio runtime using channels.
    #[instrument(
        name = "network_endpoint_bridge_outbounds",
        skip(error_tx, database, connection, receiver),
        fields(
            id = connection.id(),
            remote_address =% connection.remote_address()
        )
    )]
    fn bridge_outbounds(
        error_tx: tokio::sync::mpsc::UnboundedSender<()>,
        database: Arc<tokio::sync::Mutex<HashMap<String, std::sync::mpsc::Sender<Message>>>>,
        connection: Connection,
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkSend>,
    ) -> tokio::sync::mpsc::Sender<()> {
        let (quit_tx, mut quit_rx) = tokio::sync::mpsc::channel::<()>(1);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = receiver.recv() => {
                        tokio::spawn(Self::bridge_outbound(error_tx.clone(), database.clone(), connection.clone(), result));
                    }
                    _ = quit_rx.recv() => {
                        debug!("quit received");
                        return;
                    }
                }
            }
        });
        quit_tx
    }

    /// Bridges inbound bevy-tokio runtime using channels.
    #[instrument(
        name = "network_endpoint_bridge_inbound",
        skip(error_tx, database, sender, result)
    )]
    async fn bridge_inbound(
        error_tx: tokio::sync::mpsc::UnboundedSender<()>,
        database: Arc<tokio::sync::Mutex<HashMap<String, std::sync::mpsc::Sender<Message>>>>,
        sender: std::sync::mpsc::Sender<NetworkRecv>,
        result: Result<Message, RecvError>,
    ) {
        let message = match result {
            Ok(message) => message,
            Err(error) => {
                warn!(error =? error, "bridge error");
                if error_tx.send(()).is_err() {
                    warn!("failed to communicate error");
                }
                return;
            }
        };

        if let Some(sender) = database.lock().await.remove(&message.id) {
            if sender.send(message).is_err() {
                // the actor who sent the blocking request is no longer interested in the response.
                warn!("failed to route message to blocking channel");
            }
            return;
        }

        if sender.send(NetworkRecv::NonBlocking { message }).is_err() {
            warn!("failed to route message to non-blocking channel");
            if error_tx.send(()).is_err() {
                warn!("failed to communicate error");
            }
        }
    }

    /// Bridges outbound bevy-tokio runtime using channels.
    #[instrument(
        name = "network_endpoint_bridge_outbound",
        skip(error_tx, database, connection, result),
        fields(
            id = connection.id(),
            remote_address =% connection.remote_address()
        )
    )]
    async fn bridge_outbound(
        error_tx: tokio::sync::mpsc::UnboundedSender<()>,
        database: Arc<tokio::sync::Mutex<HashMap<String, std::sync::mpsc::Sender<Message>>>>,
        connection: Connection,
        result: Option<NetworkSend>,
    ) {
        let Some(network_send) = result else {
            warn!("bridge error");
            if error_tx.send(()).is_err() {
                warn!("failed to communicate error");
            }
            return;
        };

        let (message, blocking) = match network_send {
            NetworkSend::Blocking { message, sender } => {
                let id = message.id.clone();
                (message, Some((id, sender)))
            }
            NetworkSend::NonBlocking { message } => (message, None),
        };

        if connection.send(message).await.is_err() {
            warn!("failed to route message to connection");
            if error_tx.send(()).is_err() {
                warn!("failed to communicate error");
            }
            return;
        }

        if let Some((id, sender)) = blocking {
            database.lock().await.insert(id, sender);
        }
    }
}

/// Network Recv.
#[allow(clippy::module_name_repetitions)]
pub enum NetworkRecv {
    /// Non Blocking.
    NonBlocking {
        /// Message.
        message: Message,
    },
}

/// Network Send.
#[allow(clippy::module_name_repetitions)]
pub enum NetworkSend {
    /// Blocking.
    Blocking {
        /// Message.
        message: Message,

        /// Sender.
        sender: std::sync::mpsc::Sender<Message>,
    },

    /// Non Blocking.
    NonBlocking {
        /// Message.
        message: Message,
    },
}

/// Network Server.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct NetworkServer {
    receiver: std::sync::Mutex<std::sync::mpsc::Receiver<NetworkEndpoint>>,
}

impl NetworkServer {
    /// Creates a new [`NetworkServer`].
    fn new(receiver: std::sync::mpsc::Receiver<NetworkEndpoint>) -> Self {
        Self {
            receiver: std::sync::Mutex::new(receiver),
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

/// Connecting.
#[derive(Component)]
pub struct Connecting {
    inner: Future<Result<NetworkEndpoint, AcceptError>>,
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
    pub fn try_poll(&self) -> Poll<Result<Result<NetworkEndpoint, AcceptError>, PollError>> {
        self.inner.try_poll()
    }
}
