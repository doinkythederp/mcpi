#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms, /* missing_docs, */ clippy::missing_const_for_fn, rust_2024_compatibility)]

use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use block::{BlockFace, InvalidBlockFaceError, ParseBlockError};
use connection::commands::*;
use connection::{
    ApiStr, ChatString, ConnectOptions, ConnectionError, EntityId, NewlineStrError, Protocol,
    ServerConnection, Tile, TileData, WorldSettingKey,
};
use derive_more::derive::From;
use entity::{ClientPlayer, Player};
use futures_core::Stream;
use itertools::Itertools;
use nalgebra::{Point2, Point3};
use snafu::{OptionExt, Snafu};

pub mod block;
pub mod camera;
pub mod connection;
pub mod entity;
pub mod util;

pub use block::Block;
use tokio::net::ToSocketAddrs;
use tokio::sync::{Mutex, MutexGuard};

/// Error type for the World struct
#[derive(Debug, Snafu)]
pub enum WorldError {
    /// An error caused by interacting with a Minecraft: Pi Edition game server.
    #[snafu(display("{source}"), context(false))]
    Connection { source: ConnectionError },
    /// An error caused by creating an [`ApiStr`] that contains a LF (line feed)
    /// character.
    #[snafu(display("{source}"), context(false))]
    ApiStrConvert { source: NewlineStrError },
    /// An error caused by failing to parse an integer from a string.
    #[snafu(display("{source}"), context(false))]
    ParseInt { source: ParseIntError },
    /// An error caused by failing to parse an floating point number from a
    /// string.
    #[snafu(display("{source}"), context(false))]
    ParseFloat { source: ParseFloatError },
    /// An error caused by failing to parse a block returned by the server.
    #[snafu(display("{source}"), context(false))]
    ParseBlock { source: ParseBlockError },
    /// There was not enough data in the server's response.
    NotEnoughParts,
    /// A block face returned by the server was invalid.
    #[snafu(display("{source}"), context(false))]
    InvalidBlockFace { source: InvalidBlockFaceError },
}

pub type Result<T = (), E = WorldError> = std::result::Result<T, E>;

#[derive(Debug, From)]
pub struct World<T: Protocol = ServerConnection> {
    connection: Arc<Mutex<T>>,
}

impl<T: Protocol> Clone for World<T> {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
        }
    }
}

impl<T: Protocol> From<T> for World<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl World<ServerConnection> {
    pub async fn connect(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(Self::new(
            ServerConnection::new(addr, ConnectOptions::default()).await?,
        ))
    }
}

impl<T: Protocol> World<T> {
    pub fn new(connection: T) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
        }
    }

    pub async fn connection(&self) -> MutexGuard<'_, T> {
        self.connection.lock().await
    }

    pub async fn send_command(
        &self,
        command: impl SerializableCommand,
    ) -> Result<String, ConnectionError> {
        self.connection().await.send(command).await
    }

    /// Post one or more messages to the in-game chat as the user.
    ///
    /// Because it is not possible to send multi-line chat messages, each line
    /// (split by `\n` on all platforms) is sent individually. Messages are
    /// re-encoded into the [CP437](https://en.wikipedia.org/wiki/Code_page_437)
    /// character set, which only contains a subset of the characters available
    /// in UTF-8. Symbols that don't exist in this character set are
    /// substituted for the `?` symbol.
    #[doc(alias("chat", "say", "send"))]
    pub async fn post(&mut self, text: &str) -> Result<(), WorldError> {
        let messages = text
            .split('\n')
            .map(ChatString::from_str_lossy)
            .collect::<Vec<_>>();
        let mut conn = self.connection().await;
        for message in messages {
            conn.send(ChatPost { message }).await?;
        }
        Ok(())
    }

    /// Post a single message to the in-game chat as the user, without extra
    /// processing.
    pub async fn post_message(&mut self, message: ChatString<'_>) -> Result<(), WorldError> {
        self.send_command(ChatPost { message }).await?;
        Ok(())
    }

    /// Gets the type of the block at the given coordinates.
    pub async fn get_tile(&self, coords: Point3<i16>) -> Result<Tile> {
        let tile = self.send_command(WorldGetBlock { coords }).await?;
        Ok(tile.parse()?)
    }

    /// Gets the types and location offsets relative to `coords_0` of the blocks
    /// inclusively contained in the given cuboid.
    ///
    /// Raspberry Juice server only!
    pub async fn get_tiles(
        &self,
        coords_1: Point3<i16>,
        coords_2: Point3<i16>,
    ) -> Result<Vec<(Tile, Point3<i16>)>> {
        let blocks = self
            .send_command(raspberry_juice::WorldGetBlocks { coords_1, coords_2 })
            .await?;

        // Order: by z, then x, then y.
        let x_len = coords_2.x - coords_1.x + 1;
        let y_len = coords_2.y - coords_1.y + 1;

        let blocks = blocks
            .split(',')
            .enumerate()
            .map(|(idx, ty)| {
                let tile = Tile::from_str(ty)?;
                let idx = idx as i16;
                let z = idx / (x_len * y_len);
                let x = (idx / y_len) % x_len;
                let y = idx % y_len;

                Ok((tile, Point3::new(x, y, z)))
            })
            .collect::<Result<Vec<_>, WorldError>>()?;

        Ok(blocks)
    }

    /// Gets the type and metadata of the block at the given coordinates.
    pub async fn get_block(&self, coords: Point3<i16>) -> Result<Block> {
        Ok(self
            .send_command(WorldGetBlockWithData { coords })
            .await?
            .parse()?)
    }

    /// Sets the block at the given coordinates to the specified type.
    ///
    /// This method is shorthand for [`Self::set_block`] with `Block::new(tile,
    /// None)`.
    pub async fn set_tile(&mut self, coords: Point3<i16>, tile: Tile) -> Result<()> {
        self.send_command(WorldSetBlock {
            coords,
            tile,
            data: TileData::default(),
            json_nbt: None,
        })
        .await?;
        Ok(())
    }

    /// Sets the blocks inclusively contained in the given cuboid to the
    /// specified type.
    ///
    /// This method is shorthand for [`Self::set_blocks`] with `Block::new(tile,
    /// None)`.
    pub async fn set_tiles(
        &mut self,
        coords_1: Point3<i16>,
        coords_2: Point3<i16>,
        tile: Tile,
    ) -> Result<()> {
        self.send_command(WorldSetBlocks {
            coords_1,
            coords_2,
            tile,
            data: TileData::default(),
            json_nbt: None,
        })
        .await?;
        Ok(())
    }

    /// Updates the blocks inclusively contained in the given cuboid to have the
    /// specified type and metadata.
    pub async fn set_blocks(
        &mut self,
        coords_1: Point3<i16>,
        coords_2: Point3<i16>,
        block: &Block,
    ) -> Result<()> {
        let nbt = block.json_nbt();
        self.send_command(WorldSetBlocks {
            coords_1,
            coords_2,
            tile: block.tile,
            data: block.data,
            json_nbt: nbt.as_deref().map(ApiStr::new).transpose()?,
        })
        .await?;
        Ok(())
    }

    /// Updates the block at the given coordinates to have the specified type
    /// and metadata.
    pub async fn set_block(&mut self, coords: Point3<i16>, block: &Block) -> Result<()> {
        let nbt = block.json_nbt();
        self.send_command(WorldSetBlock {
            coords,
            tile: block.tile,
            data: block.data,
            json_nbt: nbt.as_deref().map(ApiStr::new).transpose()?,
        })
        .await?;
        Ok(())
    }

    /// Finds the Y-coordinate of the highest non-air block at the given X and Z
    /// coordinates.
    pub async fn get_height_at(&self, coords: Point2<i16>) -> Result<i16> {
        let y = self.send_command(WorldGetHeight { coords }).await?;
        Ok(y.parse()?)
    }

    /// Returns the player entity controlled by the connected game instance
    /// (i.e. the host player).
    pub fn me(&self) -> ClientPlayer<T> {
        ClientPlayer::new(self.clone())
    }

    pub async fn save_checkpoint(&mut self) -> Result<()> {
        self.send_command(WorldCheckpointSave {}).await?;
        Ok(())
    }

    pub async fn restore_checkpoint(&mut self) -> Result<()> {
        self.send_command(WorldCheckpointRestore {}).await?;
        Ok(())
    }

    /// Returns all players currently in the world.
    pub async fn all_players(&self) -> Result<Vec<Player<T>>> {
        let ids = self.send_command(WorldGetPlayerIds {}).await?;
        let players = ids
            .split('|')
            .map(|id| {
                let id = EntityId(id.parse()?);
                Ok::<_, WorldError>(Player::new(self.clone(), id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(players)
    }

    /// Enables or disables a setting that controls the behavior or the game
    /// world.
    pub async fn set(&mut self, setting: WorldSettingKey<'_>, enabled: bool) -> Result<()> {
        self.send_command(WorldSetting {
            key: setting,
            value: enabled,
        })
        .await?;
        Ok(())
    }

    /// Enables or disables world immutability.
    ///
    /// When enabled, players cannot edit the world (such as by placing or
    /// destroying blocks).
    pub async fn set_immutable(&mut self, enabled: bool) -> Result {
        self.set(WorldSettingKey::WORLD_IMMUTABLE, enabled).await
    }

    /// Enables or disables name tag visibility.
    ///
    /// When disabled, player name tags will not be shown above their heads.
    pub async fn set_nametags_visible(&mut self, enabled: bool) -> Result {
        self.set(WorldSettingKey::NAMETAGS_VISIBLE, enabled).await
    }

    /// Clears any pending events that have yet to be received.
    pub async fn clear_events(&mut self) -> Result<()> {
        self.send_command(EventsClear {}).await?;
        Ok(())
    }

    /// Polls for any block hits that have occurred since the last call to this
    /// method.
    pub async fn poll_block_hits(&self) -> Result<Vec<BlockHit>> {
        let hits = self.send_command(EventsBlockHits {}).await?;
        hits.split('|')
            .map(|hit| {
                let [x, y, z, face, player_id] = hit
                    .split(',')
                    .collect_array()
                    .context(NotEnoughPartsSnafu)?;
                Ok::<_, WorldError>(BlockHit {
                    location: Point3::new(x.parse()?, y.parse()?, z.parse()?),
                    face: face.parse::<u8>()?.try_into()?,
                    player_id: player_id.parse()?,
                })
            })
            .collect()
    }

    /// Creates a stream of block hit events. If the connection's event queue is
    /// full, polls will not be sent.
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval at which to poll for block hits.
    pub fn block_hits(&self, interval: Duration) -> impl Stream<Item = Result<BlockHit>> {
        let world = self.clone();
        async_stream::stream! {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                let hits = match world.poll_block_hits().await {
                    Ok(hits) => hits,
                    Err(e) => match e {
                        WorldError::Connection { source: ConnectionError::QueueFull { .. } } => {
                            continue;
                        }
                        e => {
                            yield Err(e);
                            return;
                        },
                    }
                };
                for hit in hits {
                    yield Ok(hit);
                }
            }
        }
    }

    /// Disconnection from the world after ensuring all pending events are sent.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.connection().await.close().await?;
        Ok(())
    }
}

/// Represents a block hit event.
///
/// Block hits are usually triggered when a player right clicks a block with a
/// sword. This may differ depending on the server implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockHit {
    /// The coordinates of the block that was hit.
    pub location: Point3<i16>,
    /// The face of the block that was hit.
    pub face: BlockFace,
    /// The ID of the player that hit the block.
    pub player_id: EntityId,
}

/// Converts the floating-point position coordinates of an entity to integer
/// tile coordinates.
///
/// # Example
///
/// ```
/// # use mcpi::pos_to_tile;
/// # use nalgebra::Point3;
/// let pos = Point3::new(1.3, 2.8, 3.4);
/// let tile = pos_to_tile(&pos);
/// assert_eq!(tile, Point3::new(1, 2, 3));
/// ``````
pub fn pos_to_tile(pos: &Point3<f64>) -> Point3<i16> {
    Point3::new(
        pos.x.floor() as i16,
        pos.y.floor() as i16,
        pos.z.floor() as i16,
    )
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
