use std::{convert::From, marker::PhantomData};

use bevy::prelude::*;
use bevy::utils::Uuid;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

/*
 * ============================================================================
 * Message
 * ============================================================================
 */

/// Message.
#[derive(Debug, Clone)]
pub struct Message<T> {
    /// Id.
    pub id: Uuid,

    /// Endpoint.
    pub endpoint: String,

    /// Header.
    pub header: MessageHeader,

    /// Payload.
    pub payload: T,
}

impl<T> From<chaos_symphony_network::Message> for Message<T>
where
    T: DeserializeOwned,
{
    fn from(value: chaos_symphony_network::Message) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            endpoint: value.endpoint,
            header: serde_json::from_str(&value.header).unwrap(),
            payload: serde_json::from_str(&value.payload).unwrap(),
        }
    }
}

impl<T> From<Message<T>> for chaos_symphony_network::Message
where
    T: Serialize,
{
    fn from(value: Message<T>) -> Self {
        Self {
            id: value.id.to_string(),
            endpoint: value.endpoint,
            header: serde_json::to_string(&value.header).unwrap(),
            payload: serde_json::to_string(&value.payload).unwrap(),
        }
    }
}

/// Message Header.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageHeader {}

/// Message ID.
#[allow(clippy::module_name_repetitions)]
pub trait MessageId {
    /// Id.
    fn id(&self) -> Uuid;
}

impl<T> MessageId for Message<T> {
    fn id(&self) -> Uuid {
        self.id
    }
}

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Authenticate Callback.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Component)]
pub struct MessageCallback<T> {
    /// Id.
    pub id: Uuid,

    future: Future<chaos_symphony_network::Message>,

    marker: PhantomData<T>,
}

impl<T> MessageCallback<T>
where
    T: From<chaos_symphony_network::Message>,
{
    /// Creates a new [`MessageCallback`].
    #[must_use]
    pub fn new(id: Uuid, future: Future<chaos_symphony_network::Message>) -> Self {
        Self {
            id,
            future,
            marker: PhantomData,
        }
    }

    /// Id.
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<T, PollError>> {
        self.future.try_poll().map(|result| result.map(Into::into))
    }
}

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Event.
pub trait Event<T>
where
    Self: Into<chaos_symphony_network::Message>,
{
    /// Endpoint.
    const ENDPOINT: &'static str;

    /// Creates a new [`Message`].
    #[must_use]
    fn message(id: Uuid, payload: T) -> Message<T> {
        Message {
            id,
            endpoint: Self::ENDPOINT.to_string(),
            header: MessageHeader {},
            payload,
        }
    }

    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    fn try_send(self, endpoint: &NetworkEndpoint) -> Result<(), SendError<NetworkSend>> {
        endpoint.try_send_non_blocking(self.into())
    }
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Request.
pub trait Request<T, U>
where
    Self: Into<chaos_symphony_network::Message> + MessageId,
    U: From<chaos_symphony_network::Message>,
{
    /// Endpoint.
    const ENDPOINT: &'static str;

    /// Creates a new [`Message`].
    #[must_use]
    fn message(id: Uuid, payload: T) -> Message<T> {
        Message {
            id,
            endpoint: Self::ENDPOINT.to_string(),
            header: MessageHeader {},
            payload,
        }
    }

    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    fn try_send(
        self,
        endpoint: &NetworkEndpoint,
    ) -> Result<MessageCallback<U>, SendError<NetworkSend>> {
        let id = self.id();
        endpoint
            .try_send_blocking(self.into())
            .map(|future| MessageCallback::<U>::new(id, future))
    }
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Response.
pub trait Response<T>
where
    Self: Into<chaos_symphony_network::Message>,
{
    /// Endpoint.
    const ENDPOINT: &'static str;

    /// Creates a new [`Message`].
    #[must_use]
    fn message(id: Uuid, payload: T) -> Message<T> {
        Message {
            id,
            endpoint: Self::ENDPOINT.to_string(),
            header: MessageHeader {},
            payload,
        }
    }

    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    fn try_send(self, endpoint: &NetworkEndpoint) -> Result<(), SendError<NetworkSend>> {
        endpoint.try_send_non_blocking(self.into())
    }
}
