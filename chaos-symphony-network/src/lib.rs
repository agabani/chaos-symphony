#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Network

use std::{fs, io, net::SocketAddr, sync::Arc};

/// Accept Error.
#[derive(Debug)]
pub enum AcceptError {
    /// Connection.
    Connection(quinn::ConnectionError),
}

/// Client.
#[derive(Debug, Clone)]
pub struct Client {
    inner: quinn::Endpoint,
}

impl Client {
    /// Creates a new [`Client`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if unable to bind to port or find certificate.
    ///
    /// # Panics
    ///
    /// Will panic if unable to parse socket address.
    pub fn new() -> Result<Self, io::Error> {
        let config = Self::config()?;
        let mut inner = quinn::Endpoint::client("[::]:0".parse().unwrap())?;
        inner.set_default_client_config(config);
        Ok(Self { inner })
    }

    /// Connect.
    ///
    /// # Errors
    ///
    /// Will return `Err` if unable to connect to server.
    ///
    /// # Panics
    ///
    /// Will panic if unable to parse socket address.
    pub fn connect(&self) -> Result<Connecting, ConnectError> {
        let inner = self
            .inner
            .connect("[::1]:4433".parse().unwrap(), "localhost")
            .map_err(ConnectError::Connect)?;
        Ok(Connecting { inner })
    }

    fn config() -> Result<quinn::ClientConfig, io::Error> {
        let dirs = directories::ProjectDirs::from("uk.co", "agabani", "chaos-symphony").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");

        let cert = fs::read(cert_path)?;
        let cert = rustls::Certificate(cert);

        let mut roots = rustls::RootCertStore::empty();
        roots.add(&cert).unwrap();

        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"hq-29".into()];

        let config = quinn::ClientConfig::new(Arc::new(config));
        Ok(config)
    }
}

/// Connect Error.
#[derive(Debug)]
pub enum ConnectError {
    /// Connect.
    Connect(quinn::ConnectError),
}

/// Connecting.
#[derive(Debug)]
pub struct Connecting {
    inner: quinn::Connecting,
}

impl Connecting {
    /// Accept.
    ///
    /// # Errors
    ///
    /// Will return `Err` if connection is lost.
    pub async fn accept(self) -> Result<Connection, AcceptError> {
        let inner = self.inner.await.map_err(AcceptError::Connection)?;
        Ok(Connection { inner })
    }

    /// Returns the remote address of this [`Connecting`].
    #[must_use]
    pub fn remote_address(&self) -> SocketAddr {
        self.inner.remote_address()
    }
}

/// Connection.
#[derive(Debug, Clone)]
pub struct Connection {
    inner: quinn::Connection,
}

impl Connection {
    /// Returns the id of this [`Connection`].
    #[must_use]
    pub fn id(&self) -> usize {
        self.inner.stable_id()
    }

    /// Recv.
    ///
    /// # Errors
    ///
    /// Will return `Err` if connection is lost or unable to read.
    pub async fn recv(&self) -> Result<Message, RecvError> {
        let (_, mut recv) = self
            .inner
            .accept_bi()
            .await
            .map_err(RecvError::Connection)?;
        let buf = recv
            .read_to_end(usize::MAX)
            .await
            .map_err(RecvError::Read)?;
        serde_json::from_slice(&buf).map_err(RecvError::Json)
    }

    /// Returns the remote address of this [`Connection`].
    #[must_use]
    pub fn remote_address(&self) -> SocketAddr {
        self.inner.remote_address()
    }

    /// Send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if connection is lost or unable to write.
    pub async fn send(&self, message: Message) -> Result<(), SendError> {
        let buf = serde_json::to_vec(&message).map_err(SendError::Json)?;
        let (mut send, _) = self.inner.open_bi().await.map_err(SendError::Connection)?;
        send.write_all(&buf).await.map_err(SendError::Write)?;
        send.finish().await.map_err(SendError::Write)?;
        Ok(())
    }
}

/// Message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Message {
    /// Id.
    pub id: String,

    /// Endpoint.
    pub endpoint: String,

    /// Header.
    pub header: String,

    /// Payload.
    pub payload: String,
}

/// Send Error.
#[derive(Debug)]
pub enum RecvError {
    /// Connection.
    Connection(quinn::ConnectionError),

    /// Json.
    Json(serde_json::Error),

    /// Read.
    Read(quinn::ReadToEndError),
}

/// Send Error.
#[derive(Debug)]
pub enum SendError {
    /// Connection.
    Connection(quinn::ConnectionError),

    /// Json.
    Json(serde_json::Error),

    /// Write.
    Write(quinn::WriteError),
}

/// Server.
#[derive(Debug, Clone)]
pub struct Server {
    inner: quinn::Endpoint,
}

impl Server {
    /// Creates a new [`Server`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if unable to bind to port or find certificate.
    ///
    /// # Panics
    ///
    /// Will panic if unable to parse socket address.
    pub fn new() -> Result<Self, io::Error> {
        let config = Self::config()?;
        let inner = quinn::Endpoint::server(config, "[::1]:4433".parse().unwrap())?;
        Ok(Self { inner })
    }

    /// Accept.
    pub async fn accept(&self) -> Option<Connecting> {
        let inner = self.inner.accept().await?;
        Some(Connecting { inner })
    }

    fn config() -> Result<quinn::ServerConfig, io::Error> {
        let dirs = directories::ProjectDirs::from("uk.co", "agabani", "chaos-symphony").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");

        let (cert, key) = match fs::read(&cert_path)
            .and_then(|cert| Ok((cert, fs::read(&key_path)?)))
        {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let cert = rcgen::generate_simple_self_signed(["localhost".to_string()]).unwrap();
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der().unwrap();
                fs::create_dir_all(path)?;
                fs::write(&cert_path, &cert)?;
                fs::write(&key_path, &key)?;
                (cert, key)
            }
            Err(error) => {
                return Err(error);
            }
        };

        let cert = rustls::Certificate(cert);
        let key = rustls::PrivateKey(key);

        let mut config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
        config.alpn_protocols = vec![b"hq-29".into()];

        let mut config = quinn::ServerConfig::with_crypto(Arc::new(config));
        let transport_config = Arc::get_mut(&mut config.transport).unwrap();
        transport_config.max_concurrent_uni_streams(0_u8.into());

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::{Client, Message, Server};

    #[tokio::test]
    async fn test_connection() {
        // Arrange
        let server = Server::new().unwrap();
        let client = Client::new().unwrap();

        tokio::spawn(async move {
            println!("server: listening");
            loop {
                let connecting = server.accept().await.unwrap();
                println!("server: connecting");
                tokio::spawn(async move {
                    let connection = connecting.accept().await.unwrap();
                    println!("server: connection");

                    loop {
                        let buf = connection.recv().await.unwrap();
                        println!("server: {buf:?}");
                        connection.send(buf).await.unwrap();
                    }
                });
            }
        });

        let connecting = client.connect().unwrap();
        println!("client: connecting");
        let connection = connecting.accept().await.unwrap();
        println!("client: connection");

        let (send, recv) = mpsc::channel();
        {
            let connection = connection.clone();
            tokio::spawn(async move {
                loop {
                    let buf = connection.recv().await.unwrap();
                    println!("client: {buf:?}");
                    send.send(buf).unwrap();
                }
            });
        }

        let message_1 = Message {
            id: "1".to_string(),
            endpoint: "/1".to_string(),
            header: "header 1".to_string(),
            payload: "payload 1".to_string(),
        };
        let message_2 = Message {
            id: "2".to_string(),
            endpoint: "/2".to_string(),
            header: "header 2".to_string(),
            payload: "payload 2".to_string(),
        };
        let message_3 = Message {
            id: "3".to_string(),
            endpoint: "/3".to_string(),
            header: "header 3".to_string(),
            payload: "payload 3".to_string(),
        };

        // Act
        connection.send(message_1.clone()).await.unwrap();
        connection.send(message_2.clone()).await.unwrap();
        connection.send(message_3.clone()).await.unwrap();

        // Assert
        tokio::task::spawn_blocking(move || {
            assert_eq!(message_1, recv.recv().unwrap());
            assert_eq!(message_2, recv.recv().unwrap());
            assert_eq!(message_3, recv.recv().unwrap());
        })
        .await
        .unwrap();
    }
}
