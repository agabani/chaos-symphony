use serde::{de::DeserializeOwned, Serialize};

/// Message.
#[derive(Clone)]
pub struct Message<T> {
    /// Id.
    pub id: String,

    /// Endpoint.
    pub endpoint: String,

    /// Payload.
    pub payload: T,
}

impl<T> From<chaos_symphony_network::Message> for Message<T>
where
    T: DeserializeOwned,
{
    fn from(value: chaos_symphony_network::Message) -> Self {
        Self {
            id: value.id,
            endpoint: value.endpoint,
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
            id: value.id,
            endpoint: value.endpoint,
            payload: serde_json::to_string(&value.payload).unwrap(),
        }
    }
}
