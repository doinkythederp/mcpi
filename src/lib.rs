use std::future::Future;

use connection::{ChatString, Command, ConnectionError, NewlineStrError, Protocol};
use snafu::Snafu;

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

pub trait World {
    /// Post one or more messages to the in-game chat as the user.
    fn post(&mut self, text: &str) -> impl Future<Output = Result<(), WorldError>>;
    /// Post a single message to the in-game chat as the user.
    fn post_message(
        &mut self,
        message: &ChatString,
    ) -> impl Future<Output = Result<(), WorldError>>;
}

impl<T: Protocol> World for T {
    async fn post(&mut self, text: &str) -> Result<(), WorldError> {
        let messages = text
            .split('\n')
            .map(ChatString::from_str_lossy)
            .collect::<Vec<_>>();
        for message in messages {
            self.send(Command::ChatPost(&message)).await?;
        }
        Ok(())
    }

    async fn post_message(&mut self, message: &ChatString) -> Result<(), WorldError> {
        self.send(Command::ChatPost(message)).await?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
