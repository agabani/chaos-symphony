#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Async

use std::sync::{mpsc::TryRecvError, Mutex};

/// Future.
pub struct Future<T> {
    receiver: Mutex<std::sync::mpsc::Receiver<T>>,
}

impl<T> Future<T> {
    /// Creates a new [`Future`].
    #[must_use]
    pub fn new(receiver: std::sync::mpsc::Receiver<T>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
        }
    }

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
    #[must_use]
    pub fn try_poll(&self) -> Poll<Result<T, PollError>> {
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

    /// Pending.
    Pending,
}

impl<T> Poll<T> {
    /// Maps a `Poll<T>` to `Poll<U>` by applying a function to a contained value.
    pub fn map<U, F>(self, f: F) -> Poll<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Ready(t) => Poll::Ready(f(t)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Poll Error.
#[derive(Debug)]
pub enum PollError {
    /// Disconnected.
    Disconnected,
}
