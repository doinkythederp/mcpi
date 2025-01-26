//! Commands can be sent to the game server to perform an action or query
//! information.
//!
//! Includes all commands supported by the vanilla Minecraft: Pi Edition game,
//! as well as commands from the following plugins, mods, or API extensions:
//!
//! - [Raspberry Juice](https://dev.bukkit.org/projects/raspberryjuice) plugin
//! - [MCPI Addons](https://github.com/Bigjango13/MCPI-Addons) mod
//! - [Raspberry Jam](https://github.com/arpruss/raspberryjammod)
//!
//! Each command struct is generally named after the API method it corresponds
//! to.

use std::fmt::{self, Display, Formatter};
use std::io::Write;

use nalgebra::{Point, Point2, Point3, Scalar};

use super::{ApiStr, ChatString, EntityId, PlayerSettingKey, Tile, TileData, WorldSettingKey};

pub mod mcpi_addons;
pub mod raspberry_jam;
pub mod raspberry_juice;

/// Values implementing this trait are commands that can be serialized and sent
/// to the Minecraft game server.
pub trait SerializableCommand: Send {
    /// Whether the specified command should wait for a response from the game
    /// server.
    const HAS_RESPONSE: bool;
    // Serializes the specified command into bytes that can be sent to the game
    // server.
    #[must_use]
    fn to_command_bytes(&self) -> Vec<u8>;
}

#[macro_export]
macro_rules! command_library {
    // Requests have a response from the server, while commands do not.
    (@packet_awaits_response req) => { true };
    (@packet_awaits_response cmd) => { false };

    {
        mod $lib_name:ident {
            $(
                $(#[$packet_meta:meta])*
                $vis:vis $packet_type:ident $packet_name:ident $(<$lt:lifetime>)? ($($fmt:tt)*) {
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
            $vis struct $packet_name $(<$lt>)? {
                $(
                    $(#[$field_meta])*
                    pub $field: $type,
                )*
            }

            impl $(<$lt>)? SerializableCommand for $packet_name $(<$lt>)? {
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
                if self.comma_if_some {
                    write!(f, ",")?;
                }
                write!(f, "{inner}")?;
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
        pub cmd EntitySetPos(
            "entity.setPos({target},{})",
            point(coords),
        ) {
            target: EntityId,
            coords: PosCoords,
        }
        pub cmd EntitySetTile(
            "entity.setTile({target},{})",
            point(coords),
        ) {
            target: EntityId,
            coords: TileCoords,
        }

        // ## Player APIs

        pub req PlayerGetPos("player.getPos()") {}
        pub req PlayerGetTile("player.getTile()") {}
        pub cmd PlayerSetPos(
            "player.setPos({})",
            point(coords),
        ) {
            coords: Point3<f64>
        }
        pub cmd PlayerSetTile(
            "player.setTile({})",
            point(coords),
        ) {
            coords: Point3<i16>
        }
        pub cmd PlayerSetting<'a>(
            "player.setting({key},{})",
            *value as i32,
        ) {
            key: PlayerSettingKey<'a>,
            value: bool,
        }

        // ## World APIs

        pub cmd WorldCheckpointRestore("world.checkpoint.restore()") {}

        pub cmd WorldCheckpointSave("world.checkpoint.save()") {}

        pub req WorldGetBlock(
            "world.getBlock({})",
            point(coords),
        ) {
            coords: Point3<i16>
        }

        /// Has Raspberry Jam mod extension to get block with NBT data. Requires a world setting to be set.
        /// TODO: look into this
        pub req WorldGetBlockWithData(
            "world.getBlockWithData({})",
            point(coords),
        ) {
            coords: Point3<i16>,
        }

        pub req WorldGetHeight(
            "world.getHeight({})",
            point(coords),
        ) {
            coords: Point2<i16>,
        }

        pub req WorldGetPlayerIds("world.getPlayerIds()") {}

        pub cmd WorldSetBlock<'a>(
            "world.setBlock({},{tile},{data}{})",
            point(coords),
            optional(json_nbt, true),
        ) {
            coords: Point3<i16>,
            tile: Tile,
            data: TileData,
            /// Raspberry Jam mod extension to set block with NBT data.
            ///
            /// Set to [`None`] when using other servers.
            json_nbt: Option<ApiStr<'a>>,
        }

        pub cmd WorldSetBlocks<'a>(
            "world.setBlocks({},{},{tile},{data}{})",
            point(coords_1),
            point(coords_2),
            optional(json_nbt, true),
        ) {
            coords_1: Point3<i16>,
            coords_2: Point3<i16>,
            tile: Tile,
            data: TileData,
            /// Raspberry Jam mod extension to add NBT data to the blocks being set.
            ///
            /// Set to [`None`] when using other servers.
            json_nbt: Option<ApiStr<'a>>,
        }
        pub cmd WorldSetting<'a>(
            "world.setting({key},{})",
            *value as i32,
        ) {
            key: WorldSettingKey<'a>,
            value: bool,
        }

        // Event APIs
        pub cmd EventsClear("events.clear()") {}
        pub req EventsBlockHits("events.block.hits()") {}
    }
);

#[derive(Debug)]
pub struct ChatPost<'a> {
    pub message: ChatString<'a>,
}

impl SerializableCommand for ChatPost<'_> {
    const HAS_RESPONSE: bool = false;
    fn to_command_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        write!(buf, "chat.post(").unwrap();
        buf.write_all(self.message.as_ref()).unwrap();
        writeln!(buf, ")").unwrap();
        buf
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::connection::JavaEntityType;

    #[test]
    fn chat_post_formatting() {
        let string = ChatString::from_str_lossy("Hello world. This is a \"quote.\" )");
        let command = ChatPost { message: string };
        assert_eq!(
            command.to_command_bytes(),
            b"chat.post(Hello world. This is a \"quote.\" ))\n"
        );
    }

    #[test]
    fn command_point_large_values() {
        let vec = Point3::new(1e100, 2.0, 3.0);
        let command = PlayerSetPos { coords: vec };
        assert_eq!(command.to_command_bytes(), b"player.setPos(10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,2,3)\n");
    }

    #[test]
    fn command_point_serializes_f64_int() {
        let vec = Point3::new(1.0, 2.0, 3.0);
        let command = PlayerSetPos { coords: vec };
        assert_eq!(command.to_command_bytes(), b"player.setPos(1,2,3)\n");
    }

    #[test]
    fn command_point_serializes_f64_real() {
        let vec = Point3::new(1.5, 2.5, 3.5);
        let command = PlayerSetPos { coords: vec };
        assert_eq!(command.to_command_bytes(), b"player.setPos(1.5,2.5,3.5)\n");
    }

    #[test]
    fn command_point_serializes_i16() {
        let vec = Point3::new(1, 2, 3);
        let command = WorldGetBlock { coords: vec };
        assert_eq!(command.to_command_bytes(), b"world.getBlock(1,2,3)\n");
    }

    #[test]
    fn command_point_serializes_i16_range() {
        let vec_1 = Point3::new(1, 2, 3);
        let vec_2 = Point3::new(4, 5, 6);
        let command = raspberry_juice::WorldGetBlocks {
            coords_1: vec_1,
            coords_2: vec_2,
        };
        assert_eq!(
            command.to_command_bytes(),
            b"world.getBlocks(1,2,3,4,5,6)\n"
        );
    }

    #[test]
    fn command_with_static_representation_serializes_to_vec() {
        let command = CameraModeSetFixed {};
        assert_eq!(command.to_command_bytes(), b"camera.mode.setFixed()\n");
    }

    mod optionals {
        use super::*;

        #[test]
        fn comma_omitted_when_last_arg_none() {
            let command = CameraModeSetFollow { target: None };
            assert_eq!(command.to_command_bytes(), b"camera.mode.setFollow()\n");
        }

        #[test]
        fn comma_omitted_when_last_arg_some() {
            let command = CameraModeSetFollow {
                target: Some(EntityId(1)),
            };
            assert_eq!(command.to_command_bytes(), b"camera.mode.setFollow(1)\n");
        }

        #[test]
        fn comma_omitted_when_first_arg_none() {
            let command = raspberry_juice::PlayerGetEntities {
                distance: 1,
                entity_type: None,
            };
            assert_eq!(command.to_command_bytes(), b"player.getEntities(1)\n");
        }

        #[test]
        fn comma_included_when_first_arg_some() {
            let command = raspberry_juice::PlayerGetEntities {
                distance: 1,
                entity_type: Some(JavaEntityType(2)),
            };
            assert_eq!(command.to_command_bytes(), b"player.getEntities(1,2)\n");
        }
    }

    // #[test]
    // fn raspberry_jam_camera_apis_have_no_mode() {
    //     let command = CameraModeSetNormal { target: None };
    //     assert_eq!(command.to_command_bytes(), b"camera.mode.setNormal()\n");
    //     let command = CameraSetNormal { target: None };
    //     assert_eq!(command.to_command_bytes(), b"camera.setNormal()\n");
    //     let command = CameraModeSetThirdPerson { target: None };
    //     assert_eq!(
    //         command.to_command_bytes(),
    //         b"camera.mode.setThirdPerson()\n"
    //     );
    //     let command = CameraSetThirdPerson { target: None };
    //     assert_eq!(command.to_command_bytes(), b"camera.setThirdPerson()\n");
    //     let command = CameraModeSetFollow { target: None };
    //     assert_eq!(command.to_command_bytes(), b"camera.mode.setFollow()\n");
    //     let command = CameraSetFollow { target: None };
    //     assert_eq!(command.to_command_bytes(), b"camera.setFollow()\n");
    // }
}
