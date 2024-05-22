use nalgebra::Point3;

use crate::connection::{Command, EntityId, Protocol};
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
        let command = match mode {
            CameraMode::Fixed => Command::CameraModeSetFixed,
            CameraMode::Follow(target) => Command::CameraModeSetFollow { target },
            CameraMode::Normal(target) => Command::CameraModeSetNormal { target },
            CameraMode::ThirdPerson(target) => Command::CameraModeSetThirdPerson { target },
        };

        self.connection.send(command).await?;
        Ok(())
    }

    pub async fn set_fixed(&mut self) -> Result {
        self.connection.send(Command::CameraModeSetFixed).await?;
        Ok(())
    }

    pub async fn set_follow(&mut self, target: Option<EntityId>) -> Result {
        self.connection
            .send(Command::CameraModeSetFollow { target })
            .await?;
        Ok(())
    }

    pub async fn set_normal(&mut self, target: Option<EntityId>) -> Result {
        self.connection
            .send(Command::CameraModeSetNormal { target })
            .await?;
        Ok(())
    }

    pub async fn set_third_person(&mut self, target: Option<EntityId>) -> Result {
        self.connection
            .send(Command::CameraModeSetThirdPerson { target })
            .await?;
        Ok(())
    }

    pub async fn set_position(&mut self, position: Point3<f64>) -> Result {
        self.connection
            .send(Command::CameraSetPos(position))
            .await?;
        Ok(())
    }
}
