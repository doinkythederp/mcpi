//! Raspberry Juice (& Raspberry Jam) Extensions
//!
//! https://dev.bukkit.org/projects/raspberryjuice

use super::*;
use crate::command_library;
use crate::connection::JavaEntityType;

command_library!(
    mod RaspberryJuice {
        // ## World APIs

        pub req WorldGetBlocks(
            "world.getBlocks({},{})",
            point(coords_1),
            point(coords_2),
        ) {
            coords_1: Point3<i16>,
            coords_2: Point3<i16>,
        }

        /// When using the Raspberry Jam mod, this can be set to [`None`] to get the current player's ID.
        pub req WorldGetPlayerId<'a>(
            "world.getPlayerId({})",
            optional(name, false),
        ) {
            name: Option<ApiStr<'a>>,
        }

        pub req WorldGetEntities(
            "world.getEntities({})",
            optional(entity_type, false),
        ) {
            entity_type: Option<JavaEntityType>,
        }

        pub cmd WorldRemoveEntity("world.removeEntity({entity_id})") {
            entity_id: EntityId,
        }

        pub cmd WorldRemoveEntities(
            "world.removeEntities({})",
            optional(entity_type, false),
        ) {
            entity_type: Option<JavaEntityType>,
        }

        pub cmd WorldSetSign<'a>(
            "world.setSign({},{tile},{data},{})",
            point(coords),
            lines
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ) {
            coords: Point3<i16>,
            tile: Tile,
            data: TileData,
            lines: Vec<ApiStr<'a>>,
        }

        pub cmd WorldSpawnEntity(
            "world.spawnEntity({},{entity_type})",
            point(coords),
        ) {
            coords: Point3<f64>,
            entity_type: JavaEntityType,
        }

        pub req WorldGetEntityTypes("world.getEntityTypes()") {}

        // ## Entity APIs

        pub req EntityGetName("entity.getName({entity_id})") {
            entity_id: EntityId,
        }

        pub req EntityGetDirection("entity.getDirection({entity_id})") {
            entity_id: EntityId,
        }

        pub cmd EntitySetDirection(
            "entity.setDirection({entity_id},{})",
            point(direction),
        ) {
            entity_id: EntityId,
            direction: Point3<f64>,
        }

        pub req EntityGetPitch("entity.getPitch({entity_id})") {
            entity_id: EntityId
        }
        pub cmd EntitySetPitch("entity.setPitch({entity_id},{pitch})") {
            entity_id: EntityId,
            pitch: f32,
        }
        pub req EntityGetRotation("entity.getRotation({entity_id})") {
            entity_id: EntityId,
        }
        pub cmd EntitySetRotation("entity.setRotation({entity_id},{rotation})") {
            entity_id: EntityId,
            rotation: f32,
        }
        pub cmd EntityEventsClear("entity.events.clear({entity_id})") {
            entity_id: EntityId,
        }
        pub req EntityEventsBlockHits("entity.events.block.hits({entity_id})") {
            entity_id: EntityId,
        }
        pub req EntityEventsChatPosts("entity.events.chat.posts({entity_id})") {
            entity_id: EntityId,
        }
        pub req EntityEventsProjectileHits("entity.events.projectile.hits({entity_id})") {
            entity_id: EntityId,
        }
        pub req EntityGetEntities(
            "entity.getEntities({target},{distance}{})",
            optional(entity_type, true),
        ) {
            target: EntityId,
            distance: i32,
            entity_type: Option<JavaEntityType>,
        }
        pub cmd EntityRemoveEntities(
            "entity.removeEntities({target},{distance}{})",
            optional(entity_type, true),
        ) {
            target: EntityId,
            distance: i32,
            entity_type: Option<JavaEntityType>,
        }

        // ## Player APIs
        pub req PlayerGetAbsPos("player.getAbsPos()") {}
        pub cmd PlayerSetAbsPos(
            "player.setAbsPos({})",
            point(coords),
        ) {
            coords: Point3<f64>,
        }
        pub cmd PlayerSetDirection(
            "player.setDirection({})",
            point(direction),
        ) {
            direction: Point3<f64>,
        }
        pub req PlayerGetDirection("player.getDirection()") {}
        pub cmd PlayerSetRotation("player.setRotation({rotation})") {
            rotation: f32,
        }
        pub req PlayerGetRotation("player.getRotation()") {}
        pub cmd PlayerSetPitch("player.setPitch({pitch})") {
            pitch: f32,
        }
        pub req PlayerGetPitch("player.getPitch()") {}
        pub cmd PlayerEventsClear("player.events.clear()") {}
        pub req PlayerEventsBlockHits("player.events.block.hits()") {}
        pub req PlayerEventsChatPosts("player.events.chat.posts()") {}
        pub req PlayerEventsProjectileHits("player.events.projectile.hits()") {}
        pub req PlayerGetEntities(
            "player.getEntities({distance}{})",
            optional(entity_type, true),
        ) {
            distance: i32,
            entity_type: Option<JavaEntityType>,
        }
        pub cmd PlayerRemoveEntities(
            "player.removeEntities({distance}{})",
            optional(entity_type, true),
        ) {
            distance: i32,
            entity_type: Option<JavaEntityType>,
        }

        // Events APIs
        pub req EventsChatPosts("events.chat.posts()") {}
        pub req EventsProjectileHits("events.projectile.hits()") {}
    }
);
