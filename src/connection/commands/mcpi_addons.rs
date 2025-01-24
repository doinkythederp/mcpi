//! MCPI Addons mod by Bigjango13
//!
//! https://github.com/Bigjango13/MCPI-Addons

use super::*;
use crate::command_library;
use crate::connection::{
    MCPIExtrasEntityType, MCPIExtrasEntityVariant, MCPIExtrasKey, MCPIExtrasParticle, SheepColor,
};

command_library!(
    mod MCPIAddons {
        // ## Custom Log APIs

        pub cmd CustomLogDebug<'a>("custom.log.debug({message})") {
            message: ApiStr<'a>,
        }

        pub cmd CustomLogInfo<'a>("custom.log.info({message})") {
            message: ApiStr<'a>,
        }

        pub cmd CustomLogWarn<'a>("custom.log.warn({message})") {
            message: ApiStr<'a>,
        }

        pub cmd CustomLogErr<'a>("custom.log.err({message})") {
            message: ApiStr<'a>,
        }

        // ## Custom Inventory APIs

        pub req CustomInventoryGetSlot("custom.inventory.getSlot()") {}

        pub cmd CustomInventoryUnsafeGive(
            "custom.inventory.give({}|{}|{})",
            id.unwrap_or(-2),
            auxillary.unwrap_or(-2),
            count.unwrap_or(-2),
        ) {
            id: Option<i32>,
            auxillary: Option<i32>,
            count: Option<i32>,
        }

        pub cmd CustomInventoryGive(
            "custom.inventory.give({}|{}|{})",
            id.unwrap_or(-2),
            auxillary.unwrap_or(-2),
            count.unwrap_or(-2),
        ) {
            id: Option<i32>,
            auxillary: Option<i32>,
            count: Option<i32>,
        }

        // ## Custom Override APIs

        pub cmd CustomOverrideReset("custom.override.reset()") {}

        pub cmd CustomOverride("custom.override({before},{after})") {
            before: Tile,
            after: Tile,
        }

        // ## Custom Key APIs

        pub cmd CustomKeyPress<'a>("custom.key.press({key})") {
            key: MCPIExtrasKey<'a>,
        }
        pub cmd CustomKeyRelease<'a>("custom.key.release({key})") {
            key: MCPIExtrasKey<'a>,
        }

        // ## Custom Username APIs

        pub req CustomUsernameAll("custom.username.all()") {}

        // ## Custom World API

        pub cmd CustomWorldParticle<'a>(
            "custom.world.particle({particle},{})",
            point(coords),
        ) {
            particle: MCPIExtrasParticle<'a>,
            coords: Point3<f32>,
        }

        pub req CustomWorldDir("custom.world.dir()") {}

        pub req CustomWorldName("custom.world.name()") {}

        pub req CustomWorldServername("custom.world.servername()") {}

        // ## Custom Player APIs

        pub req CustomPlayerGetHealth("custom.player.getHealth()") {}

        pub cmd CustomPlayerSetHealth("custom.player.setHealth({health})") {
            health: i32,
        }

        pub cmd CustomPlayerCloseGUI("custom.player.closeGUI()") {}

        pub req CustomPlayerGetGamemode("custom.player.getGamemode()") {}

        // Custom Entity APIs
        pub cmd CustomEntitySpawn(
            "custom.entity.spawn({},{},{health},{},{})",
            entity.entity,
            point(coords),
            point(direction),
            entity.value,
        ) {
            entity: MCPIExtrasEntityVariant,
            health: i32,
            coords: Point3<f32>,
            direction: Point2<f32>, // TODO: is this the most correct type?
        }

        pub cmd CustomEntitySetAge("custom.entity.setAge({entity_id},{age})") {
            entity_id: EntityId,
            age: i32,
        }

        pub cmd CustomEntitySetSheepColor("custom.entity.setSheepColor({entity_id},{color})") {
            entity_id: EntityId,
            color: SheepColor,
        }

        // ## Chat Events APIs

        pub req EventsChatPosts("events.chat.posts()") {}
        pub cmd EventsChatSize("events.chat.size({size})") {
            size: i32,
        }

        // ## Custom Reborn APIs

        pub req CustomRebornVersion("custom.reborn.version()") {}
        pub req CustomRebornFeature<'a>("custom.reborn.feature({feature_name})") {
            feature_name: ApiStr<'a>,
        }

        // ## Entity APIs

        pub req EntityGetEntities(
            "entity.getEntities({target},{distance}{})",
            optional(entity_type, true),
        ) {
            target: EntityId,
            distance: i32,
            entity_type: Option<MCPIExtrasEntityType>,
        }
        pub req EntityGetAllEntities("entity.getAllEntities()") {}
    }
);

// ## Custom Post APIs

pub struct CustomPostClient<'a> {
    pub message: ChatString<'a>,
}

impl SerializableCommand for CustomPostClient<'_> {
    const HAS_RESPONSE: bool = false;
    fn to_command_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        write!(buf, "custom.post.client(").unwrap();
        buf.write_all(self.message.as_ref()).unwrap();
        writeln!(buf, ")").unwrap();
        buf
    }
}

pub struct CustomPostNoPrefix<'a> {
    pub message: ChatString<'a>,
}

impl SerializableCommand for CustomPostNoPrefix<'_> {
    const HAS_RESPONSE: bool = false;
    fn to_command_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        write!(buf, "custom.post.noPrefix(").unwrap();
        buf.write_all(self.message.as_ref()).unwrap();
        writeln!(buf, ")").unwrap();
        buf
    }
}
