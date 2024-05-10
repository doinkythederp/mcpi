use std::collections::VecDeque;
use std::fmt::Display;
use std::net::SocketAddr;
use std::time::Duration;

use bytes::BytesMut;
use connection::{
    ChatString, Command, ConnectOptions, ConnectionError, NewlineStrError, Protocol,
    ServerConnection,
};
use snafu::{Backtrace, Snafu};

pub mod connection;
pub mod util;

/// Error type for the World struct
#[derive(Debug, Snafu)]
pub enum WorldError {
    #[snafu(display("{source}"), context(false))]
    Connection { source: ConnectionError },
    #[snafu(display("{source}"), context(false))]
    ApiStrConvert { source: NewlineStrError },
}

pub struct World<C = ServerConnection>
where
    C: Protocol,
{
    connection: C,
}

impl<C: Protocol> From<C> for World<C> {
    fn from(connection: C) -> Self {
        World { connection }
    }
}

impl World {
    pub async fn connect(addr: SocketAddr, options: ConnectOptions) -> std::io::Result<Self> {
        let connection = ServerConnection::new(addr, options).await?;
        Ok(World { connection })
    }

    pub fn into_inner(self) -> ServerConnection {
        self.connection
    }

    /// Post one or more messages to the in-game chat as the user.
    pub async fn post(&mut self, text: &str) -> Result<(), WorldError> {
        let messages = text
            .split('\n')
            .map(ChatString::from_str_lossy)
            .collect::<Vec<_>>();
        for message in messages {
            self.connection.send(Command::ChatPost(&message)).await?;
        }
        Ok(())
    }

    /// Post a single message to the in-game chat as the user.
    pub async fn post_message(&mut self, message: &ChatString) -> Result<(), WorldError> {
        self.connection.send(Command::ChatPost(message)).await?;
        Ok(())
    }
}

// struct SendRequest {
//     wait_for_response: bool,
//     data: Vec<u8>,
//     result: oneshot::Sender<Result<Vec<u8>, ConnectionError>>,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
