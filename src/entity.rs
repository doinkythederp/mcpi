use std::future::Future;

use nalgebra::Point3;

use crate::connection::{Command, EntityId, PlayerSettingKey, Protocol};
use crate::util::parse_point;
use crate::Result;

pub trait Entity {
    /// Returns the entity's ID, or None if this is the client player.
    fn entity_id(&self) -> Option<EntityId>;
    /// Gets the 3D coordinates of the entity as a floating-point Point.
    fn get_position(&self) -> impl Future<Output = Result<Point3<f64>>>;
    /// Sets the 3D coordinates of the entity as a floating-point Point.
    fn set_position(&mut self, position: Point3<f64>) -> impl Future<Output = Result>;
    /// Gets the 3D coordinates of the entity as an integer Point.
    ///
    /// If the entity is standing on a block, this can be thought of as the coordinates
    /// of that block, plus 1 in the y-axis.
    fn get_tile(&self) -> impl Future<Output = Result<Point3<i16>>>;
    /// Sets the 3D coordinates of the entity as an integer Point.
    fn set_tile(&mut self, tile: Point3<i16>) -> impl Future<Output = Result>;
}

/// A player's entity ID with a connection to their game.
///
/// This struct is used to interact with a player's entity in the game world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player<T: Protocol> {
    connection: T,
    id: EntityId,
}

impl<T: Protocol> Player<T> {
    pub const fn new(connection: T, id: EntityId) -> Self {
        Self { connection, id }
    }

    pub fn into_inner(self) -> T {
        self.connection
    }

    /// Returns the entity ID of the player.
    pub const fn id(&self) -> EntityId {
        self.id
    }
}

impl<T: Protocol> Entity for Player<T> {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.id)
    }

    async fn get_position(&self) -> Result<Point3<f64>> {
        let pos = self.connection.send(Command::EntityGetPos(self.id)).await?;
        let vec = parse_point(&pos)?;
        Ok(vec)
    }

    async fn set_position(&mut self, position: Point3<f64>) -> Result {
        self.connection
            .send(Command::EntitySetPos(self.id, position))
            .await?;
        Ok(())
    }

    async fn get_tile(&self) -> Result<Point3<i16>> {
        let tile = self
            .connection
            .send(Command::EntityGetTile(self.id))
            .await?;
        let vec = parse_point(&tile)?;
        Ok(vec)
    }

    async fn set_tile(&mut self, tile: Point3<i16>) -> Result {
        self.connection
            .send(Command::EntitySetTile(self.id, tile))
            .await?;
        Ok(())
    }
}

impl EntityId {
    /// Creates a Player instance from this entity ID, allowing interaction with the player.
    pub const fn to_player<T: Protocol>(self, connection: T) -> Player<T> {
        Player::new(connection, self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPlayer<T: Protocol> {
    connection: T,
}

impl<T: Protocol> ClientPlayer<T> {
    pub const fn new(connection: T) -> Self {
        Self { connection }
    }

    pub fn into_inner(self) -> T {
        self.connection
    }

    /// Enables or disables a setting that controls the behavior or the host player.
    pub async fn set(&mut self, setting: PlayerSettingKey<'_>, enabled: bool) -> Result {
        self.connection
            .send(Command::PlayerSetting {
                key: setting,
                value: enabled,
            })
            .await?;
        Ok(())
    }

    /// Enables or disables the auto-jump setting of the host player.
    ///
    /// When enabled, the player will automatically jump when walking into a block.
    pub async fn set_autojump(&mut self, enabled: bool) -> Result {
        self.set(PlayerSettingKey::AUTOJUMP, enabled).await
    }
}

impl<T: Protocol> Entity for ClientPlayer<T> {
    fn entity_id(&self) -> Option<EntityId> {
        None
    }

    async fn get_position(&self) -> Result<Point3<f64>> {
        let pos = self.connection.send(Command::PlayerGetPos).await?;
        let vec = parse_point(&pos)?;
        Ok(vec)
    }

    async fn set_position(&mut self, position: Point3<f64>) -> Result {
        self.connection
            .send(Command::PlayerSetPos(position))
            .await?;
        Ok(())
    }

    async fn get_tile(&self) -> Result<Point3<i16>> {
        let tile = self.connection.send(Command::PlayerGetTile).await?;
        let vec = parse_point(&tile)?;
        Ok(vec)
    }

    async fn set_tile(&mut self, tile: Point3<i16>) -> Result {
        self.connection.send(Command::PlayerSetTile(tile)).await?;
        Ok(())
    }
}
