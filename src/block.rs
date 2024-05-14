use std::str::FromStr;

use snafu::{OptionExt, Snafu};

use crate::connection::{Tile, TileData};
use crate::{Result, WorldError};

/// A block type and its associated data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub tile: Tile,
    pub data: TileData,
    pub nbt: Option<serde_json::Value>,
}

impl Block {
    pub const fn new(tile: Tile, data: TileData) -> Self {
        Self {
            tile,
            data,
            nbt: None,
        }
    }

    pub fn with_nbt(mut self, nbt: serde_json::Value) -> Self {
        self.nbt = Some(nbt);
        self
    }

    pub fn json_nbt(&self) -> Option<String> {
        self.nbt.as_ref().map(|v| v.to_string())
    }
}

#[derive(Debug, Snafu)]
pub enum ParseBlockError {
    NotEnoughParts,
    ParseInt,
}

impl FromStr for Block {
    type Err = WorldError;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.splitn(2, ',');
        let tile = parts.next().context(NotEnoughPartsSnafu)?.parse()?;
        let data = parts.next().context(NotEnoughPartsSnafu)?.parse()?;

        Ok(Self {
            tile: Tile(tile),
            data: TileData(data),
            nbt: None,
        })
    }
}

/// Failed to convert a block face ID to a `BlockFace`.
#[derive(Debug, Snafu)]
#[snafu(display("Invalid block face `{id}`"))]
pub struct InvalidBlockFaceError {
    id: i32,
}

/// Represents a face of a block.
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    /// The side of the block facing towards Y = -∞ (i.e. the bottom of the block).
    NegativeY = 0,
    /// The side of the block facing towards Y = ∞ (i.e. the top of the block).
    PositiveY,
    /// The side of the block facing towards z = -∞.
    NegativeZ,
    /// The side of the block facing towards Z = ∞.
    PositiveZ,
    /// The side of the block facing towards X = -∞.
    NegativeX,
    /// The side of the block facing towards X = ∞.
    PositiveX,
}

impl TryFrom<i16> for BlockFace {
    type Error = InvalidBlockFaceError;

    fn try_from(id: i16) -> Result<Self, InvalidBlockFaceError> {
        Ok(match id {
            0 => Self::NegativeY,
            1 => Self::PositiveY,
            2 => Self::NegativeZ,
            3 => Self::PositiveZ,
            4 => Self::NegativeX,
            5 => Self::PositiveX,
            id => InvalidBlockFaceSnafu { id }.fail()?,
        })
    }
}
