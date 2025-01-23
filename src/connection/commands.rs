//! A command that can be sent to the game server to perform an action or query information.
//!
//! Includes all commands supported by the vanilla Minecraft: Pi Edition game, as well as
//! commands from the following plugins, mods, or API extensions:
//!
//! - [Raspberry Juice](https://dev.bukkit.org/projects/raspberryjuice) plugin
//! - [MCPI Addons](https://github.com/Bigjango13/MCPI-Addons) mod
//! - [Raspberry Jam](https://github.com/arpruss/raspberryjammod)
//!
//! Enum members are generally named after the API method they correspond to, with the exception of
//! a few extension commands that have conflicting names.

use std::fmt::{self, Display, Formatter};
use std::io::Write;

use nalgebra::{Point, Point3, Scalar};

use super::{ChatString, EntityId, PlayerSettingKey};

/// Values implementing this trait are commands that can be serialized and sent to the Minecraft
/// game server.
pub trait SerializableCommand {
    /// Whether the specified command should wait for a response from the game server.
    const HAS_RESPONSE: bool;
    // Serializes the specified command into bytes that can be sent to the game server.
    #[must_use]
    fn to_command_bytes(&self) -> Vec<u8>;
}

macro_rules! command_library {
    // Requests have a response from the server, while commands do not.
    (@packet_awaits_response req) => { true };
    (@packet_awaits_response cmd) => { false };

    {
        mod $lib_name:ident {
            $(
                $(#[$packet_meta:meta])*
                $vis:vis $packet_type:ident $packet_name:ident ($($fmt:tt)*) {
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident : $type:ty
                    ),*
                    $(,)?
                }
            )*
        }
    } => {
        $(
            #[derive(Debug)]
            $(#[$packet_meta])*
            $vis struct $packet_name {
                $(
                    $(#[$field_meta])*
                    pub $field: $type,
                )*
            }

            impl SerializableCommand for $packet_name {
                const HAS_RESPONSE: bool = command_library!(@packet_awaits_response $packet_type);
                fn to_command_bytes(&self) -> Vec<u8> {
                    let mut buf = Vec::new();
                    let Self {
                        $(
                            $field,
                        )*
                    } = &self;
                    writeln!(buf, $($fmt)*).unwrap();
                    return buf;
                }
            }
        )*
    };
}

/// A helper for command libraries that displays an empty string
/// when its inner field is empty.
pub fn optional<T: Display>(param: &Option<T>, comma_if_some: bool) -> impl Display + '_ {
    struct MaybeField<'a, T: Display> {
        param: &'a Option<T>,
        comma_if_some: bool,
    }

    impl<T: Display> Display for MaybeField<'_, T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            if let Some(inner) = &self.param {
                write!(f, "{inner}")?;
                if self.comma_if_some {
                    write!(f, ",")?;
                }
            }
            Ok(())
        }
    }

    MaybeField {
        param,
        comma_if_some,
    }
}

pub fn point<T: Display + Scalar, const D: usize>(param: &Point<T, D>) -> String {
    param
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub type PosCoords = Point3<f64>;
pub type TileCoords = Point3<i16>;

// # Vanilla Commands

command_library!(
    mod Vanilla {
        // ## Camera APIs

        pub cmd CameraModeSetFixed("camera.mode.setFixed()") {}

        pub cmd CameraModeSetFollow(
            "camera.mode.setFollow({})",
            optional(target, false),
        ) {
            target: Option<EntityId>,
        }

        pub cmd CameraModeSetNormal(
            "camera.mode.setNormal({})",
            optional(target, false),
        ) {
            target: Option<EntityId>,
        }

        // TODO: Test whether this works on vanilla
        pub cmd CameraModeSetThirdPerson(
            "camera.mode.setThirdPerson({})",
            optional(target, false),
        ) {
            target: Option<EntityId>,
        }

        pub cmd CameraSetPos(
            "camera.setPos({})",
            point(coords),
        ) {
            coords: PosCoords,
        }

        // ## Entity APIs

        pub req EntityGetPos("entity.getPos({target})") {
            target: EntityId,
        }
        pub req EntityGetTile("entity.getTile({target}") {
            target: EntityId,
        }
        pub cmd EntitySetPos("entity.setPos({target},{coords})") {
            target: EntityId,
            coords: PosCoords,
        }
        pub cmd EntitySetTile("entity.setTile({target},{coords})") {
            target: EntityId,
            coords: TileCoords,
        }

        // ## Player APIs

        pub req PlayerGetPos("player.getPos()") {}
        pub req PlayerGetTile("player.getTile()") {}
        pub cmd PlayerSetPos("player.setTile({coords})") {
            coords: Point3<f64>
        }
        pub cmd PlayerSetTile("player.setTile({coords})") {
            coords: Point3<i16>
        }
        pub cmd PlayerSetting(
            "player.setting({key},{})",
            *value as i32,
        ) {
            key: PlayerSettingKey,
            value: bool,
        }
    }
);

#[derive(Debug)]
pub struct ChatPost {
    pub message: ChatString,
}

impl SerializableCommand for ChatPost {
    const HAS_RESPONSE: bool = false;
    fn to_command_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        writeln!(buf, "chat.post(").unwrap();
        buf.write_all(self.message.as_ref()).unwrap();
        writeln!(buf, ")").unwrap();
        buf
    }
}

// Camera APIs

// // Chat APIs
// // Entity APIs
// EntityGetPos(EntityId),
// EntityGetTile(EntityId),
// EntitySetPos(EntityId, Point3<f64>),
// EntitySetTile(EntityId, Point3<i16>),
// // Player APIs
// PlayerGetPos,
// PlayerGetTile,
// PlayerSetPos(Point3<f64>),
// PlayerSetTile(Point3<i16>),
// PlayerSetting {
//     key: PlayerSettingKey<'a>,
//     value: bool,
// },
// // World APIs
// WorldCheckpointRestore,
// WorldCheckpointSave,
// WorldGetBlock(Point3<i16>),
// /// Has Raspberry Jam mod extension to get block with NBT data. Requires a world setting to be set.
// /// TODO: look into this
// WorldGetBlockWithData(Point3<i16>),
// WorldGetHeight(Point2<i16>),
// WorldGetPlayerIds,
// WorldSetBlock {
//     coords: Point3<i16>,
//     tile: Tile,
//     data: TileData,
//     /// Raspberry Jam mod extension to set block with NBT data.
//     ///
//     /// Set to [`None`] when using other servers.
//     json_nbt: Option<ApiStr<'a>>,
// },
// WorldSetBlocks {
//     coords_1: Point3<i16>,
//     coords_2: Point3<i16>,
//     tile: Tile,
//     data: TileData,
//     /// Raspberry Jam mod extension to add NBT data to the blocks being set.
//     ///
//     /// Set to [`None`] when using other servers.
//     json_nbt: Option<ApiStr<'a>>,
// },
// WorldSetting {
//     key: WorldSettingKey<'a>,
//     value: bool,
// },
// // Event APIs
// EventsClear,
// EventsBlockHits,

// // # Raspberry Juice (& Raspberry Jam) Extensions
// // https://dev.bukkit.org/projects/raspberryjuice

// // World APIs
// WorldGetBlocks(Point3<i16>, Point3<i16>),
// /// When using the Raspberry Jam mod, this can be set to [`None`] to get the current player's ID.
// WorldGetPlayerId(Option<ApiStr<'a>>),
// WorldGetEntities(Option<JavaEntityType>),
// WorldRemoveEntity(EntityId),
// WorldRemoveEntities(Option<JavaEntityType>),
// WorldSetSign {
//     coords: Point3<i16>,
//     tile: Tile,
//     data: TileData,
//     lines: Vec<ApiStr<'a>>,
// },
// RaspberryJuiceWorldSpawnEntity {
//     coords: Point3<f64>,
//     entity_type: JavaEntityType,
// },
// WorldGetEntityTypes,

// // Entity APIs
// EntityGetName(EntityId),
// EntityGetDirection(EntityId),
// EntitySetDirection(EntityId, Point3<f64>),
// EntityGetPitch(EntityId),
// EntitySetPitch(EntityId, f32),
// EntityGetRotation(EntityId),
// EntitySetRotation(EntityId, f32),
// EntityEventsClear(EntityId),
// EntityEventsBlockHits(EntityId),
// EntityEventsChatPosts(EntityId),
// EntityEventsProjectileHits(EntityId),
// EntityGetEntities {
//     target: EntityId,
//     distance: i32,
//     entity_type: Option<JavaEntityType>,
// },
// EntityRemoveEntities {
//     target: EntityId,
//     distance: i32,
//     entity_type: Option<JavaEntityType>,
// },

// // Player APIs
// PlayerGetAbsPos,
// PlayerSetAbsPos(Point3<f64>),
// PlayerSetDirection(Point3<f64>),
// PlayerGetDirection,
// PlayerSetRotation(f32),
// PlayerGetRotation,
// PlayerSetPitch(f32),
// PlayerGetPitch,
// PlayerEventsClear,
// PlayerEventsBlockHits,
// PlayerEventsChatPosts,
// PlayerEventsProjectileHits,
// PlayerGetEntities {
//     distance: i32,
//     entity_type: Option<JavaEntityType>,
// },
// PlayerRemoveEntities {
//     distance: i32,
//     entity_type: Option<JavaEntityType>,
// },

// // Events APIs
// EventsChatPosts,
// EventsProjectileHits,

// // # MCPI Addons mod by Bigjango13
// // https://github.com/Bigjango13/MCPI-Addons

// // Custom Log APIs
// CustomLogDebug(ApiStr<'a>),
// CustomLogInfo(ApiStr<'a>),
// CustomLogWarn(ApiStr<'a>),
// CustomLogErr(ApiStr<'a>),

// // Custom Inventory APIs
// CustomInventoryGetSlot,
// CustomInventoryUnsafeGive {
//     id: Option<i32>,
//     auxillary: Option<i32>,
//     count: Option<i32>,
// },
// CustomInventoryGive {
//     id: Option<i32>,
//     auxillary: Option<i32>,
//     count: Option<i32>,
// },

// // Custom Override APIs
// CustomOverrideReset,
// CustomOverride {
//     before: Tile,
//     after: Tile,
// },

// // Custom Post APIs
// CustomPostClient(ApiStr<'a>),
// CustomPostNoPrefix(ApiStr<'a>),

// // Custom Key APIs
// CustomKeyPress(MCPIExtrasKey<'a>),
// CustomKeyRelease(MCPIExtrasKey<'a>),

// // Custom Username APIs
// CustomUsernameAll,

// // Custom World API
// CustomWorldParticle {
//     particle: MCPIExtrasParticle<'a>,
//     coords: Point3<f32>,
// },
// CustomWorldDir,
// CustomWorldName,
// CustomWorldServername,

// // Custom Player APIs
// CustomPlayerGetHealth,
// CustomPlayerSetHealth(i32),
// CustomPlayerCloseGUI,
// CustomPlayerGetGamemode,

// // Custom Entity APIs
// CustomEntitySpawn {
//     entity_type: MCPIExtrasEntityType,
//     health: i32,
//     coords: Point3<f32>,
//     direction: Point2<f32>, // TODO: is this the most correct type?
// },
// CustomEntitySetAge {
//     entity_id: EntityId,
//     age: i32,
// },
// CustomEntitySetSheepColor {
//     entity_id: EntityId,
//     color: SheepColor,
// },

// // Chat Events APIs
// EventsChatSize,

// // Custom Reborn APIs
// CustomRebornVersion,
// CustomRebornFeature(ApiStr<'a>),

// // Entity APIs
// EntityGetAllEntities,

// // # Raspbery Jam mod
// // https://github.com/arpruss/raspberryjammod

// // World APIs
// /// Has extension to get block with NBT data. Requires a world setting to be set.
// WorldGetBlocksWithData {
//     coords_1: Point3<i16>,
//     coords_2: Point3<i16>,
// },
// WorldSpawnParticle {
//     particle: RaspberryJamParticle<'a>,
//     coords: Point3<f64>,
//     direction: Point3<f64>, // TODO: Unclear how to use this
//     speed: f64,
//     count: i32,
// },

// // Block APIs
// BlockGetLightLevel {
//     tile: Tile,
// },
// BlockSetLightLevel {
//     tile: Tile,
//     level: f32,
// },

// // Entity APIs
// EntitySetDimension {
//     entity_id: EntityId,
//     dimension: Dimension,
// },
// EntityGetNameAndUUID(EntityId),
// RaspberryJamWorldSpawnEntity {
//     entity_type: JavaEntityType,
//     coords: Point3<f64>,
//     json_nbt: Option<ApiStr<'a>>,
// },

// // Player APIs
// PlayerSetDimension {
//     dimension: Dimension,
// },
// PlayerGetNameAndUUID,

// // Camera APIs
// CameraGetEntityId,
// CameraSetFollow {
//     target: Option<EntityId>,
// },
// CameraSetNormal {
//     target: Option<EntityId>,
// },
// CameraSetThirdPerson {
//     target: Option<EntityId>,
// },
// CameraSetDebug,
// CameraSetDistance(f32),
