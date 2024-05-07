use std::collections::VecDeque;
use std::fmt::Display;
use std::net::SocketAddr;
use std::time::Duration;

use bytes::BytesMut;
use snafu::{Backtrace, Snafu};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::oneshot;

pub mod connection;

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
