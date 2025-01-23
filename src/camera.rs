use nalgebra::Point3;

use crate::connection::commands::*;
use crate::connection::{EntityId, Protocol};
use crate::Result;

pub enum CameraMode {
    Fixed,
    Follow(Option<EntityId>),
    Normal(Option<EntityId>),
    ThirdPerson(Option<EntityId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Camera<T: Protocol> {
    connection: T,
}

impl<T: Protocol> Camera<T> {
    pub const fn new(connection: T) -> Self {
        Self { connection }
    }

    pub async fn set_mode(&mut self, mode: CameraMode) -> Result {
        match mode {
            CameraMode::Fixed => self.set_fixed().await?,
            CameraMode::Follow(target) => self.set_follow(target).await?,
            CameraMode::Normal(target) => self.set_normal(target).await?,
            CameraMode::ThirdPerson(target) => self.set_third_person(target).await?,
        };
        Ok(())
    }

    pub async fn set_fixed(&mut self) -> Result {
        self.connection.send(CameraModeSetFixed {}).await?;
        Ok(())
    }

    pub async fn set_follow(&mut self, target: Option<EntityId>) -> Result {
        self.connection.send(CameraModeSetFollow { target }).await?;
        Ok(())
    }

    pub async fn set_normal(&mut self, target: Option<EntityId>) -> Result {
        self.connection.send(CameraModeSetNormal { target }).await?;
        Ok(())
    }

    pub async fn set_third_person(&mut self, target: Option<EntityId>) -> Result {
        self.connection
            .send(CameraModeSetThirdPerson { target })
            .await?;
        Ok(())
    }

    pub async fn set_position(&mut self, position: Point3<f64>) -> Result {
        self.connection
            .send(CameraSetPos { coords: position })
            .await?;
        Ok(())
    }
}
