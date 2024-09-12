use snafu::OptionExt;
use tokio::sync::{mpsc, oneshot};

use super::{
    ConnectOptions, ConnectionError, Protocol, QueueFullSnafu, SendSnafu, SerializableCommand,
    ServerConnection,
};

enum QueueItem {
    Request {
        request: Vec<u8>,
        has_response: bool,
        response: oneshot::Sender<Result<String, ConnectionError>>,
    },
    Options {
        options: ConnectOptions,
    },
    Close,
}

async fn worker(
    mut connection: ServerConnection,
    mut rx: mpsc::Receiver<QueueItem>,
) -> Result<(), ConnectionError> {
    while let Some(item) = rx.recv().await {
        match item {
            QueueItem::Request {
                request,
                has_response,
                response,
            } => {
                let result = connection.send_raw(&request, has_response).await;
                _ = response.send(result);
            }
            QueueItem::Options { options } => {
                connection.set_options(options).unwrap(); // ServerConnection never errors here
            }
            QueueItem::Close => {
                return connection.close().await;
            }
        }
    }
    _ = connection.close().await;
    Ok(())
}

/// Handle to a background task that sends requests to the server.
///
/// This struct can be cheaply cloned and sent between threads, and commands
/// sent to the server are queued up and processed in the background.
///
/// When the last handle for a connection is dropped, the queue will be depleted
/// and the connection will be closed.
#[derive(Debug, Clone)]
pub struct QueuedConnection {
    channel: mpsc::Sender<QueueItem>,
}

impl QueuedConnection {
    /// Creates a new connection to the server with a queue of the given size.
    pub async fn new(
        addr: &str,
        options: ConnectOptions,
        queue_size: usize,
    ) -> std::io::Result<Self> {
        let connection = ServerConnection::new(addr, options).await?;
        Ok(Self::from_connection(connection, queue_size).await)
    }

    /// Starts a background task that sends requests to the server.
    pub async fn from_connection(connection: ServerConnection, queue_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(queue_size);
        tokio::spawn(worker(connection, rx));
        Self { channel: tx }
    }
}

impl Protocol for QueuedConnection {
    fn pressure(&self) -> f64 {
        self.channel.capacity() as f64 - self.channel.max_capacity() as f64
    }

    fn set_options(&mut self, options: ConnectOptions) -> Result<(), ConnectionError> {
        self.channel
            .try_send(QueueItem::Options { options })
            .ok()
            .context(QueueFullSnafu)
    }

    async fn send(
        &self,
        command: impl SerializableCommand + Send,
    ) -> Result<String, ConnectionError> {
        let permit = self.channel.reserve().await.ok().context(SendSnafu)?;
        let (tx, rx) = oneshot::channel();
        let request = QueueItem::Request {
            request: command.to_command_bytes(),
            has_response: command.has_response(),
            response: tx,
        };
        permit.send(request);
        rx.await?
    }

    async fn close(self) -> Result<(), ConnectionError> {
        _ = self.channel.send(QueueItem::Close).await;
        Ok(())
    }
}
