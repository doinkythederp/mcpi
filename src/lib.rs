#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms, /* missing_docs, */ clippy::missing_const_for_fn)]

use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use block::{BlockFace, InvalidBlockFaceError, ParseBlockError};
use connection::commands::*;
use connection::{
    ApiStr, ChatString, ClonableConnection, ConnectionError, EntityId, NewlineStrError, Protocol,
    ServerConnection, Tile, TileData, WorldSettingKey,
};
// use entity::{ClientPlayer, Player};
use futures_core::Stream;
use nalgebra::{Point2, Point3};
use snafu::{OptionExt, Snafu};

pub mod block;
pub mod camera;
pub mod connection;
// pub mod entity;
pub mod util;

pub use block::Block;
use tokio::sync::Mutex;

/// Error type for the World struct
#[derive(Debug, Snafu)]
pub enum WorldError {
    #[snafu(display("{source}"), context(false))]
    Connection { source: ConnectionError },
    #[snafu(display("{source}"), context(false))]
    ApiStrConvert { source: NewlineStrError },
    #[snafu(display("{source}"), context(false))]
    ParseInt { source: ParseIntError },
    #[snafu(display("{source}"), context(false))]
    ParseFloat { source: ParseFloatError },
    #[snafu(display("{source}"), context(false))]
    ParseBlock { source: ParseBlockError },
    #[snafu(display("Not enough parts in server's response"))]
    NotEnoughParts,
    #[snafu(display("{source}"), context(false))]
    InvalidBlockFace { source: InvalidBlockFaceError },
}

pub type Result<T = (), E = WorldError> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World<T: Protocol + Clone = ClonableConnection> {
    connection: T,
}

impl<T: Protocol + Clone> From<T> for World<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl World<ClonableConnection> {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let connection = ServerConnection::new(addr, Default::default()).await?;
        Ok(Self::new(Arc::new(Mutex::new(Some(connection)))))
    }
}

impl<T: Protocol + Clone> World<T> {
    pub const fn new(connection: T) -> Self {
        Self { connection }
    }

    /// Post one or more messages to the in-game chat as the user.
    #[doc(alias("chat", "say", "send"))]
    pub async fn post(&mut self, text: &str) -> Result<(), WorldError> {
        let messages = text
            .split('\n')
            .map(ChatString::from_str_lossy)
            .collect::<Vec<_>>();
        for message in messages {
            self.connection.send(ChatPost { message }).await?;
        }
        Ok(())
    }

    /// Post a single message to the in-game chat as the user.
    pub async fn post_message(&mut self, message: ChatString) -> Result<(), WorldError> {
        self.connection.send(ChatPost { message }).await?;
        Ok(())
    }

    // /// Gets the type of the block at the given coordinates.
    // pub async fn get_tile(&self, coords: Point3<i16>) -> Result<Tile> {
    //     let ty = self.connection.send(WorldGetBlock(coords)).await?;
    //     Ok(Tile(ty.parse()?))
    // }

    // /// Gets the types and location offsets relative to `coords_0` of the blocks inclusively contained in the given cuboid.
    // ///
    // /// Raspberry Juice server only!
    // pub async fn get_tiles(
    //     &self,
    //     coords_1: Point3<i16>,
    //     coords_2: Point3<i16>,
    // ) -> Result<Vec<(Tile, Point3<i16>)>> {
    //     let blocks = self
    //         .connection
    //         .send(WorldGetBlocks(coords_1, coords_2))
    //         .await?;

    //     // Order: by z, then x, then y.
    //     let x_len = coords_2.x - coords_1.x + 1;
    //     let y_len = coords_2.y - coords_1.y + 1;

    //     let blocks = blocks
    //         .split(',')
    //         .enumerate()
    //         .map(|(idx, ty)| {
    //             let tile = Tile(ty.parse()?);
    //             let idx = idx as i16;
    //             let z = idx / (x_len * y_len);
    //             let x = (idx / y_len) % x_len;
    //             let y = idx % y_len;

    //             Ok::<_, WorldError>((tile, Point3::new(x, y, z)))
    //         })
    //         .collect::<Result<Vec<_>, _>>()?;

    //     Ok(blocks)
    // }

    // /// Gets the type and metadata of the block at the given coordinates.
    // pub async fn get_block(&self, coords: Point3<i16>) -> Result<Block> {
    //     self.connection
    //         .send(WorldGetBlockWithData(coords))
    //         .await?
    //         .parse()
    // }

    // /// Sets the block at the given coordinates to the specified type.
    // ///
    // /// This method is shorthand for [`Self::set_block`] with `Block::new(tile, None)`.
    // pub async fn set_tile(&mut self, coords: Point3<i16>, tile: Tile) -> Result<()> {
    //     self.connection
    //         .send(WorldSetBlock {
    //             coords,
    //             tile,
    //             data: TileData::default(),
    //             json_nbt: None,
    //         })
    //         .await?;
    //     Ok(())
    // }

    // /// Sets the blocks inclusively contained in the given cuboid to the specified type.
    // ///
    // /// This method is shorthand for [`Self::set_blocks`] with `Block::new(tile, None)`.
    // pub async fn set_tiles(
    //     &mut self,
    //     coords_1: Point3<i16>,
    //     coords_2: Point3<i16>,
    //     tile: Tile,
    // ) -> Result<()> {
    //     self.connection
    //         .send(WorldSetBlocks {
    //             coords_1,
    //             coords_2,
    //             tile,
    //             data: TileData::default(),
    //             json_nbt: None,
    //         })
    //         .await?;
    //     Ok(())
    // }

    // /// Updates the blocks inclusively contained in the given cuboid to have the specified type and metadata.
    // pub async fn set_blocks(
    //     &mut self,
    //     coords_1: Point3<i16>,
    //     coords_2: Point3<i16>,
    //     block: &Block,
    // ) -> Result<()> {
    //     let nbt = block.json_nbt();
    //     self.connection
    //         .send(WorldSetBlocks {
    //             coords_1,
    //             coords_2,
    //             tile: block.tile,
    //             data: block.data,
    //             json_nbt: nbt.as_deref().map(ApiStr::new).transpose()?,
    //         })
    //         .await?;
    //     Ok(())
    // }

    // /// Updates the block at the given coordinates to have the specified type and metadata.
    // pub async fn set_block(&mut self, coords: Point3<i16>, block: &Block) -> Result<()> {
    //     let nbt = block.json_nbt();
    //     self.connection
    //         .send(WorldSetBlock {
    //             coords,
    //             tile: block.tile,
    //             data: block.data,
    //             json_nbt: nbt.as_deref().map(ApiStr::new).transpose()?,
    //         })
    //         .await?;
    //     Ok(())
    // }

    // /// Finds the Y-coordinate of the highest non-air block at the given X and Z coordinates.
    // pub async fn get_height_at(&self, coords: Point2<i16>) -> Result<i16> {
    //     let y = self.connection.send(WorldGetHeight(coords)).await?;
    //     Ok(y.parse()?)
    // }

    // /// Returns the player entity controlled by the connected game instance (i.e. the host player).
    // pub fn me(&self) -> ClientPlayer<T> {
    //     ClientPlayer::new(self.connection.clone())
    // }

    // pub async fn save_checkpoint(&mut self) -> Result<()> {
    //     self.connection.send(WorldCheckpointSave).await?;
    //     Ok(())
    // }

    // pub async fn restore_checkpoint(&mut self) -> Result<()> {
    //     self.connection.send(WorldCheckpointRestore).await?;
    //     Ok(())
    // }

    // /// Returns all players currently in the world.
    // pub async fn all_players(&self) -> Result<Vec<Player<T>>> {
    //     let ids = self.connection.send(WorldGetPlayerIds).await?;
    //     let players = ids
    //         .split('|')
    //         .map(|id| {
    //             let id = EntityId(id.parse()?);
    //             Ok::<_, WorldError>(Player::new(self.connection.clone(), id))
    //         })
    //         .collect::<Result<Vec<_>, _>>()?;
    //     Ok(players)
    // }

    // /// Enables or disables a setting that controls the behavior or the game world.
    // pub async fn set(&mut self, setting: WorldSettingKey, enabled: bool) -> Result<()> {
    //     self.connection
    //         .send(WorldSetting {
    //             key: setting,
    //             value: enabled,
    //         })
    //         .await?;
    //     Ok(())
    // }

    // /// Enables or disables world immutability.
    // ///
    // /// When enabled, players cannot edit the world (such as by placing or destroying blocks).
    // pub async fn set_immutable(&mut self, enabled: bool) -> Result {
    //     self.set(WorldSettingKey::WORLD_IMMUTABLE, enabled).await
    // }

    // /// Enables or disables name tag visibility.
    // ///
    // /// When disabled, player name tags will not be shown above their heads.
    // pub async fn set_nametags_visible(&mut self, enabled: bool) -> Result {
    //     self.set(WorldSettingKey::NAMETAGS_VISIBLE, enabled).await
    // }

    // /// Clears any pending events that have yet to be received.
    // pub async fn clear_events(&mut self) -> Result<()> {
    //     self.connection.send(EventsClear).await?;
    //     Ok(())
    // }

    // /// Polls for any block hits that have occurred since the last call to this method.
    // pub async fn poll_block_hits(&self) -> Result<Vec<BlockHit>> {
    //     let hits = self.connection.send(EventsBlockHits).await?;
    //     hits.split('|')
    //         .map(|hit| {
    //             let mut hit = hit.split(',').map(i16::from_str);
    //             Ok::<_, WorldError>(BlockHit {
    //                 coords: Point3::new(
    //                     hit.next().context(NotEnoughPartsSnafu)??,
    //                     hit.next().context(NotEnoughPartsSnafu)??,
    //                     hit.next().context(NotEnoughPartsSnafu)??,
    //                 ),
    //                 face: hit.next().context(NotEnoughPartsSnafu)??.try_into()?,
    //                 player_id: EntityId(hit.next().context(NotEnoughPartsSnafu)??.into()),
    //             })
    //         })
    //         .collect()
    // }

    // /// Creates a stream of block hit events. If the connection's event queue is full, polls will not be sent.
    // ///
    // /// # Arguments
    // ///
    // /// * `interval` - The interval at which to poll for block hits.
    // pub fn block_hits(&self, interval: Duration) -> impl Stream<Item = Result<BlockHit>> {
    //     let world = self.clone();
    //     async_stream::stream! {
    //         let mut interval = tokio::time::interval(interval);
    //         loop {
    //             interval.tick().await;
    //             let hits = match world.poll_block_hits().await {
    //                 Ok(hits) => hits,
    //                 Err(e) => match e {
    //                     WorldError::Connection { source: ConnectionError::QueueFull { .. } } => {
    //                         continue;
    //                     }
    //                     e => {
    //                         yield Err(e);
    //                         return;
    //                     },
    //                 }
    //             };
    //             for hit in hits {
    //                 yield Ok(hit);
    //             }
    //         }
    //     }
    // }

    /// Disconnection from the world after ensuring all pending events are sent.
    pub async fn disconnect(self) -> Result<()> {
        self.connection.close().await?;
        Ok(())
    }
}

/// Represents a block hit event.
///
/// Block hits are usually triggered when a player right clicks a block with a sword.
/// This may differ depending on your server implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockHit {
    /// The coordinates of the block that was hit.
    pub coords: Point3<i16>,
    /// The face of the block that was hit.
    pub face: BlockFace,
    /// The ID of the player that hit the block.
    pub player_id: EntityId,
}

/// Converts the floating-point position coordinates of an entity to integer tile coordinates.
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
