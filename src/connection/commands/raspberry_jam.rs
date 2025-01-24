//! Raspberry Jam mod
//!
//! https://github.com/arpruss/raspberryjammod

use super::*;
use crate::command_library;
use crate::connection::{Dimension, JavaEntityType, RaspberryJamParticle};

command_library!(
    mod RaspberryJam {
        // ## World APIs

        /// Has extension to get block with NBT data. Requires a world setting to be set.
        pub req WorldGetBlocksWithData(
            "world.getBlocksWithData({},{})",
            point(coords_1),
            point(coords_2),
        ) {
            coords_1: Point3<i16>,
            coords_2: Point3<i16>,
        }

        pub cmd WorldSpawnParticle<'a>(
            "world.spawnParticle({particle},{},{},{speed},{count})",
            point(coords),
            point(direction),
        ) {
            particle: RaspberryJamParticle<'a>,
            coords: Point3<f64>,
            direction: Point3<f64>, // TODO: Unclear how to use this
            speed: f64,
            count: i32,
        }

        // ## Block APIs

        pub req BlockGetLightLevel("block.getLightLevel({tile})") {
            tile: Tile,
        }

        pub cmd BlockSetLightLevel("block.setLightLevel({tile},{level})") {
            tile: Tile,
            level: f32,
        }

        // ## Entity APIs

        pub cmd EntitySetDimension("entity.setDimension({entity_id},{dimension})") {
            entity_id: EntityId,
            dimension: Dimension,
        }

        pub req EntityGetNameAndUUID("entity.getNameAndUUID({entity_id})") {
            entity_id: EntityId
        }

        pub cmd WorldSpawnEntity<'a>(
            "world.spawnEntity({entity_type},{}{})",
            point(coords),
            optional(json_nbt, true),
        ) {
            entity_type: JavaEntityType,
            coords: Point3<f64>,
            json_nbt: Option<ApiStr<'a>>,
        }

        // ## Player APIs
        pub cmd PlayerSetDimension("player.setDimension({dimension})") {
            dimension: Dimension,
        }

        pub req PlayerGetNameAndUUID("player.getNameAndUUID()") {}

        // ## Camera APIs
        pub req CameraGetEntityId("camera.getEntityId()") {}

        pub cmd CameraSetFollow(
            "camera.setFollow({})",
            optional(target, false),
        ) {
            target: Option<EntityId>,
        }

        pub cmd CameraSetNormal(
            "camera.setNormal({})",
            optional(target, false),
        ) {
            target: Option<EntityId>,
        }

        pub cmd CameraSetThirdPerson(
            "camera.setThirdPerson({})",
            optional(target, false),
        ) {
            target: Option<EntityId>,
        }

        pub cmd CameraSetDebug("camera.setDebug()") {}

        pub cmd CameraSetDistance("camera.setDistance({distance})") {
            distance: f32,
        }
    }
);
