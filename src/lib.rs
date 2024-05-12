#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms, /* missing_docs, */ clippy::missing_const_for_fn)]

use std::future::Future;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

use block::ParseBlockError;
use connection::queued::QueuedConnection;
use connection::{
    ApiStr, ChatString, Command, ConnectionError, EntityId, NewlineStrError, Protocol, Tile,
    TileData, WorldSettingKey,
};
use entity::{ClientPlayer, Player};
use nalgebra::{DMatrix, Vector2, Vector3};
use snafu::Snafu;

pub mod block;
pub mod camera;
pub mod connection;
pub mod entity;
pub mod util;

pub use block::Block;

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
}

pub(crate) type Result<T = (), E = WorldError> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World<T: Protocol> {
    connection: T,
}

impl<T: Protocol> From<T> for World<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl World<QueuedConnection> {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let connection = QueuedConnection::new(addr, Default::default(), 100).await?;
        Ok(Self::new(connection))
    }
}

impl<T: Protocol> World<T> {
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
            self.connection.send(Command::ChatPost(&message)).await?;
        }
        Ok(())
    }

    /// Post a single message to the in-game chat as the user.
    pub async fn post_message(&mut self, message: &ChatString) -> Result<(), WorldError> {
        self.connection.send(Command::ChatPost(message)).await?;
        Ok(())
    }

    /// Gets the type of the block at the given coordinates.
    pub async fn get_tile(&self, coords: Vector3<i16>) -> Result<Tile> {
        let ty = self.connection.send(Command::WorldGetBlock(coords)).await?;
        Ok(Tile(ty.parse()?))
    }

    /// Gets the types and location offsets relative to `coords_0` of the blocks inclusively contained in the given cuboid.
    ///
    /// Raspberry Juice server only!
    pub async fn get_tiles(
        &self,
        coords_1: Vector3<i16>,
        coords_2: Vector3<i16>,
    ) -> Result<Vec<(Tile, Vector3<i16>)>> {
        let blocks = self
            .connection
            .send(Command::WorldGetBlocks(coords_1, coords_2))
            .await?;

        // Order: by z, then x, then y.
        let x_len = coords_2.x - coords_1.x + 1;
        let y_len = coords_2.y - coords_1.y + 1;

        let blocks = blocks
            .split(',')
            .enumerate()
            .map(|(idx, ty)| {
                let tile = Tile(ty.parse()?);
                let idx = idx as i16;
                let z = idx / (x_len * y_len);
                let x = (idx / y_len) % x_len;
                let y = idx % y_len;

                Ok::<_, WorldError>((tile, Vector3::new(x, y, z)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(blocks)
    }

    /// Gets the type and metadata of the block at the given coordinates.
    pub async fn get_block(&self, coords: Vector3<i16>) -> Result<Block> {
        self.connection
            .send(Command::WorldGetBlockWithData(coords))
            .await?
            .parse()
    }

    /// Sets the block at the given coordinates to the specified type.
    ///
    /// This method is shorthand for [`Self::set_block`] with `Block::new(tile, None)`.
    pub async fn set_tile(&mut self, coords: Vector3<i16>, tile: Tile) -> Result<()> {
        self.connection
            .send(Command::WorldSetBlock {
                coords,
                tile,
                data: TileData::default(),
                json_nbt: None,
            })
            .await?;
        Ok(())
    }

    /// Sets the blocks inclusively contained in the given cuboid to the specified type.
    ///
    /// This method is shorthand for [`Self::set_blocks`] with `Block::new(tile, None)`.
    pub async fn set_tiles(
        &mut self,
        coords_1: Vector3<i16>,
        coords_2: Vector3<i16>,
        tile: Tile,
    ) -> Result<()> {
        self.connection
            .send(Command::WorldSetBlocks {
                coords_1,
                coords_2,
                tile,
                data: TileData::default(),
                json_nbt: None,
            })
            .await?;
        Ok(())
    }

    /// Updates the blocks inclusively contained in the given cuboid to have the specified type and metadata.
    pub async fn set_blocks(
        &mut self,
        coords_1: Vector3<i16>,
        coords_2: Vector3<i16>,
        block: &Block,
    ) -> Result<()> {
        let nbt = block.json_nbt();
        self.connection
            .send(Command::WorldSetBlocks {
                coords_1,
                coords_2,
                tile: block.tile,
                data: block.data,
                json_nbt: nbt.as_deref().map(ApiStr::new).transpose()?,
            })
            .await?;
        Ok(())
    }

    /// Updates the block at the given coordinates to have the specified type and metadata.
    pub async fn set_block(&mut self, coords: Vector3<i16>, block: &Block) -> Result<()> {
        let nbt = block.json_nbt();
        self.connection
            .send(Command::WorldSetBlock {
                coords,
                tile: block.tile,
                data: block.data,
                json_nbt: nbt.as_deref().map(ApiStr::new).transpose()?,
            })
            .await?;
        Ok(())
    }

    /// Finds the Y-coordinate of the highest non-air block at the given X and Z coordinates.
    pub async fn get_height_at(&self, coords: Vector2<i16>) -> Result<i16> {
        let y = self
            .connection
            .send(Command::WorldGetHeight(coords))
            .await?;
        Ok(y.parse()?)
    }

    /// Returns the player entity controlled by the connected game instance (i.e. the host player).
    pub fn me(&self) -> ClientPlayer<T> {
        ClientPlayer::new(self.connection.clone())
    }

    pub async fn save_checkpoint(&mut self) -> Result<()> {
        self.connection.send(Command::WorldCheckpointSave).await?;
        Ok(())
    }

    pub async fn restore_checkpoint(&mut self) -> Result<()> {
        self.connection
            .send(Command::WorldCheckpointRestore)
            .await?;
        Ok(())
    }

    /// Returns all players currently in the world.
    pub async fn all_players(&self) -> Result<Vec<Player<T>>> {
        let ids = self.connection.send(Command::WorldGetPlayerIds).await?;
        let players = ids
            .split('|')
            .map(|id| {
                let id = EntityId(id.parse()?);
                Ok::<_, WorldError>(Player::new(self.connection.clone(), id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(players)
    }

    /// Enables or disables a setting that controls the behavior or the game world.
    pub async fn set(&mut self, setting: WorldSettingKey<'_>, enabled: bool) -> Result<()> {
        self.connection
            .send(Command::WorldSetting {
                key: setting,
                value: enabled,
            })
            .await?;
        Ok(())
    }

    /// Enables or disables world immutability.
    ///
    /// When enabled, players cannot edit the world (such as by placing or destroying blocks).
    pub async fn set_immutable(&mut self, enabled: bool) -> Result {
        self.set(WorldSettingKey::WORLD_IMMUTABLE, enabled).await
    }

    /// Enables or disables name tag visibility.
    ///
    /// When disabled, player name tags will not be shown above their heads.
    pub async fn set_nametags_visible(&mut self, enabled: bool) -> Result {
        self.set(WorldSettingKey::NAMETAGS_VISIBLE, enabled).await
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
