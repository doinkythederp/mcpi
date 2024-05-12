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
