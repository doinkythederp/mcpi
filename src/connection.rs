//! Implementation of the Minecraft: Pi Edition protocol.
//!
//! Sources include:
//! - [Picraft docs](https://picraft.readthedocs.io/en/release-1.0/protocol.html)
//! - [Wiki.vg](https://wiki.vg/Minecraft_Pi_Protocol)
//! - [martinohanlon/Minecraft-Pi-API](https://github.com/martinohanlon/Minecraft-Pi-API/blob/master/api.md)
//! - [MCPI Revival Wiki](https://mcpirevival.miraheze.org/wiki/MCPI_Revival)

use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::str::FromStr;
use std::time::Duration;

use commands::SerializableCommand;
use derive_more::derive::{Constructor, FromStr};
use derive_more::{AsRef, Display};
use snafu::{Backtrace, OptionExt, Snafu};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::oneshot::error::RecvError;
use tokio::time::error::Elapsed;
use tokio::time::timeout;

use crate::util::{Cp437String, CHAR_TO_CP437};

// MARK: Enums

/// A block that can be used in Minecraft: Pi Edition.
///
/// Vanilla blocks are available as associated constants.
///
/// See also: [Minecraft: Pi Edition Complete Block List](https://mcpirevival.miraheze.org/wiki/Minecraft:_Pi_Edition_Complete_Block_List)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, FromStr)]
pub struct Tile(pub u8);

impl Tile {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const GRASS_BLOCK: Self = Self(2);
    pub const DIRT: Self = Self(3);
    pub const COBBLESTONE: Self = Self(4);
    pub const PLANKS: Self = Self(5);
    pub const SAPLING: Self = Self(6);
    pub const BEDROCK: Self = Self(7);
    pub const WATER: Self = Self(8);
    pub const STILL_WATER: Self = Self(9);
    pub const LAVA: Self = Self(10);
    pub const STILL_LAVA: Self = Self(11);
    pub const SAND: Self = Self(12);
    pub const GRAVEL: Self = Self(13);
    pub const GOLD_ORE: Self = Self(14);
    pub const IRON_ORE: Self = Self(15);
    pub const COAL_ORE: Self = Self(16);
    pub const LOG: Self = Self(17);
    pub const LEAVES: Self = Self(18);
    pub const GLASS: Self = Self(20);
    pub const LAPIS_ORE: Self = Self(21);
    pub const LAPIS_BLOCK: Self = Self(22);
    pub const SANDSTONE: Self = Self(24);
    pub const BED: Self = Self(26);
    pub const COBWEB: Self = Self(30);
    pub const BUSH: Self = Self(31);
    pub const WOOL: Self = Self(35);
    pub const DANDELION: Self = Self(37);
    pub const BLUE_ROSE: Self = Self(38);
    pub const BROWN_MUSHROOM: Self = Self(39);
    pub const RED_MUSHROOM: Self = Self(40);
    pub const GOLD_BLOCK: Self = Self(41);
    pub const IRON_BLOCK: Self = Self(42);
    pub const DOUBLE_SLAB: Self = Self(43);
    pub const SLAB: Self = Self(44);
    pub const BRICKS: Self = Self(45);
    pub const TNT: Self = Self(46);
    pub const BOOKSHELF: Self = Self(47);
    pub const MOSSY_COBBLESTONE: Self = Self(48);
    pub const OBSIDIAN: Self = Self(49);
    pub const TORCH: Self = Self(50);
    pub const FIRE: Self = Self(51);
    pub const WOODEN_STAIRS: Self = Self(53);
    pub const CHEST: Self = Self(54);
    pub const DIAMOND_ORE: Self = Self(56);
    pub const DIAMOND_BLOCK: Self = Self(57);
    pub const CRAFTING_TABLE: Self = Self(58);
    pub const WHEAT: Self = Self(59);
    pub const FARMLAND: Self = Self(60);
    pub const FURNACE: Self = Self(61);
    pub const LIT_FURNACE: Self = Self(62);
    pub const SIGN: Self = Self(63);
    pub const WOODEN_DOOR: Self = Self(64);
    /// This tile is invisible by default and requires TileData to be made
    /// visible.
    pub const LADDER: Self = Self(65);
    pub const COBBLESTONE_STAIRS: Self = Self(67);
    pub const WALL_SIGN: Self = Self(68);
    pub const IRON_DOOR: Self = Self(71);
    pub const REDSTONE_ORE: Self = Self(73);
    pub const LIT_REDSTONE_ORE: Self = Self(74);
    pub const SNOW: Self = Self(78);
    pub const ICE: Self = Self(79);
    pub const SNOW_BLOCK: Self = Self(80);
    pub const CACTUS: Self = Self(81);
    pub const CLAY: Self = Self(82);
    pub const SUGARCANE: Self = Self(83);
    pub const FENCE: Self = Self(85);
    pub const NETHERRACK: Self = Self(87);
    pub const GLOWSTONE: Self = Self(89);
    #[doc(alias = "BARRIER")]
    pub const INVISIBLE_BEDROCK: Self = Self(95);
    pub const TRAPDOOR: Self = Self(96);
    pub const STONE_BRICKS: Self = Self(98);
    pub const GLASS_PANE: Self = Self(102);
    pub const MELON: Self = Self(103);
    pub const MELON_STEM: Self = Self(105);
    pub const FENCE_GATE: Self = Self(107);
    pub const BRICK_STAIRS: Self = Self(108);
    pub const STONE_BRICK_STAIRS: Self = Self(109);
    pub const NETHER_BRICKS: Self = Self(112);
    pub const NETHER_BRICK_STAIRS: Self = Self(114);
    pub const SANDSTONE_STAIRS: Self = Self(128);
    pub const QUARTZ: Self = Self(155);
    pub const QUARTZ_STAIRS: Self = Self(156);
    pub const STONECUTTER: Self = Self(245);
    pub const GLOWING_OBSIDIAN: Self = Self(246);
    pub const NETHER_REACTOR_CORE: Self = Self(247);
    pub const UPDATE: Self = Self(248);
    pub const ATEUPD: Self = Self(249);
    pub const GRASS_BLOCK_CARRIED: Self = Self(253);
    /// This tile is a darker version of [`LEAVES`].
    pub const LEAVES_CARRIED: Self = Self(254);
    /// This tile is a duplicate of [`STONE`] with a different ID.
    pub const STONE_1: Self = Self(255);

    pub const fn display(&self) -> TileDisplay {
        TileDisplay(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef)]
pub struct TileDisplay(Tile);

impl Display for TileDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self.as_ref() {
            Tile::AIR => write!(f, "Air"),
            Tile::STONE => write!(f, "Stone"),
            Tile::GRASS_BLOCK => write!(f, "Grass Block"),
            Tile::DIRT => write!(f, "Dirt"),
            Tile::COBBLESTONE => write!(f, "Cobblestone"),
            Tile::PLANKS => write!(f, "Planks"),
            Tile::SAPLING => write!(f, "Sapling"),
            Tile::BEDROCK => write!(f, "Bedrock"),
            Tile::WATER => write!(f, "Water"),
            Tile::STILL_WATER => write!(f, "Still Water"),
            Tile::LAVA => write!(f, "Lava"),
            Tile::STILL_LAVA => write!(f, "Still Lava"),
            Tile::SAND => write!(f, "Sand"),
            Tile::GRAVEL => write!(f, "Gravel"),
            Tile::GOLD_ORE => write!(f, "Gold Ore"),
            Tile::IRON_ORE => write!(f, "Iron Ore"),
            Tile::COAL_ORE => write!(f, "Coal Ore"),
            Tile::LOG => write!(f, "Log"),
            Tile::LEAVES => write!(f, "Leaves"),
            Tile::GLASS => write!(f, "Glass"),
            Tile::LAPIS_ORE => write!(f, "Lapis Ore"),
            Tile::LAPIS_BLOCK => write!(f, "Lapis Block"),
            Tile::SANDSTONE => write!(f, "Sandstone"),
            Tile::BED => write!(f, "Bed"),
            Tile::COBWEB => write!(f, "Cobweb"),
            Tile::BUSH => write!(f, "Bush"),
            Tile::WOOL => write!(f, "Wool"),
            Tile::DANDELION => write!(f, "Dandelion"),
            Tile::BLUE_ROSE => write!(f, "Blue Rose"),
            Tile::BROWN_MUSHROOM => write!(f, "Brown Mushroom"),
            Tile::RED_MUSHROOM => write!(f, "Red Mushroom"),
            Tile::GOLD_BLOCK => write!(f, "Gold Block"),
            Tile::IRON_BLOCK => write!(f, "Iron Block"),
            Tile::DOUBLE_SLAB => write!(f, "Double Slab"),
            Tile::SLAB => write!(f, "Slab"),
            Tile::BRICKS => write!(f, "Bricks"),
            Tile::TNT => write!(f, "TNT"),
            Tile::BOOKSHELF => write!(f, "Bookshelf"),
            Tile::MOSSY_COBBLESTONE => write!(f, "Mossy Cobblestone"),
            Tile::OBSIDIAN => write!(f, "Obsidian"),
            Tile::TORCH => write!(f, "Torch"),
            Tile::FIRE => write!(f, "Fire"),
            Tile::WOODEN_STAIRS => write!(f, "Wooden Stairs"),
            Tile::CHEST => write!(f, "Chest"),
            Tile::DIAMOND_ORE => write!(f, "Diamond Ore"),
            Tile::DIAMOND_BLOCK => write!(f, "Diamond Block"),
            Tile::CRAFTING_TABLE => write!(f, "Crafting Table"),
            Tile::WHEAT => write!(f, "Wheat"),
            Tile::FARMLAND => write!(f, "Farmland"),
            Tile::FURNACE => write!(f, "Furnace"),
            Tile::LIT_FURNACE => write!(f, "Lit Furnace"),
            Tile::SIGN => write!(f, "Sign"),
            Tile::WOODEN_DOOR => write!(f, "Wooden Door"),
            Tile::LADDER => write!(f, "Ladder"),
            Tile::COBBLESTONE_STAIRS => write!(f, "Cobblestone Stairs"),
            Tile::WALL_SIGN => write!(f, "Wall Sign"),
            Tile::IRON_DOOR => write!(f, "Iron Door"),
            Tile::REDSTONE_ORE => write!(f, "Redstone Ore"),
            Tile::LIT_REDSTONE_ORE => write!(f, "Lit Redstone Ore"),
            Tile::SNOW => write!(f, "Snow"),
            Tile::ICE => write!(f, "Ice"),
            Tile::SNOW_BLOCK => write!(f, "Snow Block"),
            Tile::CACTUS => write!(f, "Cactus"),
            Tile::CLAY => write!(f, "Clay"),
            Tile::SUGARCANE => write!(f, "Sugarcane"),
            Tile::FENCE => write!(f, "Fence"),
            Tile::NETHER_BRICKS => write!(f, "Nether Bricks"),
            Tile::GLOWSTONE => write!(f, "Glowstone"),
            Tile::INVISIBLE_BEDROCK => write!(f, "Invisible Bedrock"),
            Tile::TRAPDOOR => write!(f, "Trapdoor"),
            Tile::STONE_BRICKS => write!(f, "Stone Bricks"),
            Tile::GLASS_PANE => write!(f, "Glass Pane"),
            Tile::MELON => write!(f, "Melon"),
            Tile::MELON_STEM => write!(f, "Melon Stem"),
            Tile::FENCE_GATE => write!(f, "Fence Gate"),
            Tile::BRICK_STAIRS => write!(f, "Brick Stairs"),
            Tile::STONE_BRICK_STAIRS => write!(f, "Stone Brick Stairs"),
            Tile::NETHER_BRICK_STAIRS => write!(f, "Nether Brick Stairs"),
            Tile::SANDSTONE_STAIRS => write!(f, "Sandstone Stairs"),
            Tile::QUARTZ => write!(f, "Quartz"),
            Tile::QUARTZ_STAIRS => write!(f, "Quartz Stairs"),
            Tile::STONECUTTER => write!(f, "Stonecutter"),
            Tile::GLOWING_OBSIDIAN => write!(f, "Glowing Obsidian"),
            Tile::NETHER_REACTOR_CORE => write!(f, "Nether Reactor Core"),
            Tile::UPDATE => write!(f, "Update Block"),
            Tile::ATEUPD => write!(f, "Ateupd Block"),
            Tile::GRASS_BLOCK_CARRIED => write!(f, "Grass Block Carried"),
            Tile::LEAVES_CARRIED => write!(f, "Leaves Carried"),
            Tile::STONE_1 => write!(f, "Stone 1"),
            _ => write!(f, "Unknown Block ({})", self.0),
        }
    }
}

/// Extra data that can be attached to a block, specific to that block type.
///
/// For many blocks, this data is used to represent the block's state, such as
/// growth stage or orientation. These common values are available as associated
/// constants.
///
/// When working with blocks that don't store any extra state, the TileData will
/// not be used by the server, but can be set and later retrieved by the API
/// user.
///
/// See also: [Minecraft: Pi Edition Complete Block List](https://mcpirevival.miraheze.org/wiki/Minecraft:_Pi_Edition_Complete_Block_List)
#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, FromStr,
)]
#[as_ref(forward)]
pub struct TileData(pub u8);

impl TileData {
    // Used with blocks that do not have tile data
    pub const NONE: Self = Self(0);

    // LOG, LEAVES, SAPLING
    pub const OAK: Self = Self(0);
    pub const SPRUCE: Self = Self(1);
    pub const BIRCH: Self = Self(2);
    // WATER, STILL_WATER, LAVA, STILL_WATER
    pub const LIQUID_FULL: Self = Self(0);
    pub const LIQUID_7: Self = Self(1);
    pub const LIQUID_6: Self = Self(2);
    pub const LIQUID_5: Self = Self(3);
    pub const LIQUID_4: Self = Self(4);
    pub const LIQUID_3: Self = Self(5);
    pub const LIQUID_2: Self = Self(6);
    pub const LIQUID_1: Self = Self(7);
    pub const LIQUID_FLOWING_DOWN: Self = Self(8);
    // SANDSTONE
    pub const SANDSTONE_NORMAL: Self = Self(0);
    pub const SANDSTONE_CHISELLED: Self = Self(1);
    pub const SANDSTONE_SMOOTH: Self = Self(1);
    // BED
    pub const BED_BOTTOM_Z_POSITIVE: Self = Self(0);
    pub const BED_BOTTOM_X_POSITIVE: Self = Self(1);
    pub const BED_BOTTOM_Z_NEGATIVE: Self = Self(2);
    pub const BED_BOTTOM_X_NEGATIVE: Self = Self(3);
    pub const BED_TOP_Z_POSITIVE: Self = Self(8);
    pub const BED_TOP_X_POSITIVE: Self = Self(9);
    pub const BED_TOP_Z_NEGATIVE: Self = Self(10);
    pub const BED_TOP_X_NEGATIVE: Self = Self(11);
    // BUSH
    pub const BUSH_DEAD: Self = Self(0);
    pub const BUSH_GRASS: Self = Self(1);
    pub const BUSH_FERN: Self = Self(3);
    // WOOL
    pub const WHITE: Self = Self(0);
    pub const ORANGE: Self = Self(1);
    pub const MAGENTA: Self = Self(2);
    pub const LIGHT_BLUE: Self = Self(3);
    pub const YELLOW: Self = Self(4);
    pub const LIME: Self = Self(5);
    pub const PINK: Self = Self(6);
    pub const GRAY: Self = Self(7);
    pub const LIGHT_GRAY: Self = Self(8);
    pub const CYAN: Self = Self(9);
    pub const PURPLE: Self = Self(10);
    pub const BLUE: Self = Self(11);
    pub const BROWN: Self = Self(12);
    pub const GREEN: Self = Self(13);
    pub const RED: Self = Self(14);
    pub const BLACK: Self = Self(15);
    // SLAB, DOUBLE_SLAB
    pub const SLAB_STONE: Self = Self(0);
    pub const SLAB_SANDSTONE: Self = Self(1);
    pub const SLAB_WOOD: Self = Self(2);
    pub const SLAB_COBBLESTONE: Self = Self(3);
    pub const SLAB_BRICKS: Self = Self(4);
    pub const SLAB_STONE_BRICKS: Self = Self(5);
    pub const SLAB_POLISHED_STONE: Self = Self(6);
    pub const SLAB_STONE_TOP: Self = Self(8);
    pub const SLAB_SANDSTONE_TOP: Self = Self(9);
    pub const SLAB_WOOD_TOP: Self = Self(10);
    pub const SLAB_COBBLESTONE_TOP: Self = Self(11);
    pub const SLAB_BRICKS_TOP: Self = Self(12);
    pub const SLAB_STONE_BRICKS_TOP: Self = Self(13);
    pub const SLAB_POLISHED_STONE_TOP: Self = Self(14);
    // TNT
    pub const TNT_INACTIVE: Self = Self(0);
    pub const TNT_ACTIVE: Self = Self(1);
    // WOODEN_STAIRS, COBBLESTONE_STAIRS, BRICK_STAIRS,
    // STONE_BRICK_STAIRS, NETHER_BRICK_STAIRS, SANDSTONE_STAIRS,
    // QUARTZ_STAIRS
    pub const STAIRS_X_POSITIVE: Self = Self(0);
    pub const STAIRS_X_NEGATIVE: Self = Self(1);
    pub const STAIRS_Z_POSITIVE: Self = Self(2);
    pub const STAIRS_Z_NEGATIVE: Self = Self(3);
    pub const STAIRS_X_POSITIVE_UPSIDE_DOWN: Self = Self(4);
    pub const STAIRS_X_NEGATIVE_UPSIDE_DOWN: Self = Self(5);
    pub const STAIRS_Z_POSITIVE_UPSIDE_DOWN: Self = Self(6);
    pub const STAIRS_Z_NEGATIVE_UPSIDE_DOWN: Self = Self(7);
    // CHEST
    pub const CHEST_NOT_FACING: Self = Self(0);
    pub const CHEST_Z_NEGATIVE: Self = Self(2);
    pub const CHEST_Z_POSITIVE: Self = Self(3);
    pub const CHEST_X_NEGATIVE: Self = Self(4);
    pub const CHEST_X_POSITIVE: Self = Self(5);
    // MELON_STEM, WHEAT
    pub const GROWTH_STAGE_0: Self = Self(0);
    pub const GROWTH_STAGE_1: Self = Self(1);
    pub const GROWTH_STAGE_2: Self = Self(2);
    pub const GROWTH_STAGE_3: Self = Self(3);
    pub const GROWTH_STAGE_4: Self = Self(4);
    pub const GROWTH_STAGE_5: Self = Self(5);
    pub const GROWTH_STAGE_6: Self = Self(6);
    pub const GROWTH_STAGE_7: Self = Self(7);
    // WHEAT only
    pub const GROWTH_STAGE_LEVER: Self = Self(8);
    pub const GROWTH_STAGE_DOOR: Self = Self(9);
    pub const GROWTH_STAGE_IRON_DOOR: Self = Self(10);
    pub const GROWTH_STAGE_REDSTONE_TORCH: Self = Self(11);
    pub const GROWTH_STAGE_MOSSY_STONE_BRICKS: Self = Self(12);
    pub const GROWTH_STAGE_CRACKED_STONE_BRICKS: Self = Self(13);
    pub const GROWTH_STAGE_PUMPKIN: Self = Self(14);
    pub const GROWTH_STAGE_NETHERRACK: Self = Self(15);
    // SIGN
    pub const SIGN_Z_POSITIVE: Self = Self(0);
    pub const SIGN_Z_POSITIVE_POSITIVE_X_NEGATIVE: Self = Self(1);
    pub const SIGN_Z_POSITIVE_X_NEGATIVE: Self = Self(2);
    pub const SIGN_Z_POSITIVE_X_NEGATIVE_NEGATIVE: Self = Self(3);
    pub const SIGN_X_NEGATIVE: Self = Self(4);
    pub const SIGN_X_NEGATIVE_NEGATIVE_Z_NEGATIVE: Self = Self(5);
    pub const SIGN_X_NEGATIVE_Z_NEGATIVE: Self = Self(6);
    pub const SIGN_X_NEGATIVE_Z_NEGATIVE_NEGATIVE: Self = Self(7);
    pub const SIGN_Z_NEGATIVE: Self = Self(8);
    pub const SIGN_Z_NEGATIVE_NEGATICE_X_POSITIVE: Self = Self(9);
    pub const SIGN_Z_NEGATIVE_X_POSITIVE: Self = Self(10);
    pub const SIGN_Z_NEGATIVE_X_POSITIVE_POSITIVE: Self = Self(11);
    pub const SIGN_X_POSITIVE: Self = Self(12);
    pub const SIGN_X_POSITIVE_POSITIVE_Z_POSITIVE: Self = Self(13);
    pub const SIGN_X_POSITIVE_Z_POSITIVE: Self = Self(14);
    pub const SIGN_X_POSITIVE_Z_POSITIVE_POSITIVE: Self = Self(15);
    // WOODEN_DOOR, IRON_DOOR
    pub const DOOR_OPENED_BOTTOM_X_NEGATIVE: Self = Self(0);
    pub const DOOR_OPENED_BOTTOM_Z_NEGATIVE: Self = Self(1);
    pub const DOOR_OPENED_BOTTOM_X_POSITIVE: Self = Self(2);
    pub const DOOR_OPENED_BOTTOM_Z_POSITIVE: Self = Self(3);
    pub const DOOR_CLOSED_BOTTOM_X_NEGATIVE: Self = Self(4);
    pub const DOOR_CLOSED_BOTTOM_Z_NEGATIVE: Self = Self(5);
    pub const DOOR_CLOSED_BOTTOM_X_POSITIVE: Self = Self(6);
    pub const DOOR_CLOSED_BOTTOM_Z_POSITIVE: Self = Self(7);
    pub const DOOR_CLOSED_TOP_X_NEGATIVE: Self = Self(8);
    // TRAPDOOR
    pub const TRAPDOOR_CLOSED_Z_POSITIVE: Self = Self(0);
    pub const TRAPDOOR_CLOSED_Z_NEGATIVE: Self = Self(1);
    pub const TRAPDOOR_CLOSED_X_POSITIVE: Self = Self(2);
    pub const TRAPDOOR_CLOSED_X_NEGATIVE: Self = Self(3);
    pub const TRAPDOOR_OPENED_Z_POSITIVE: Self = Self(4);
    pub const TRAPDOOR_OPENED_Z_NEGATIVE: Self = Self(5);
    pub const TRAPDOOR_OPENED_X_POSITIVE: Self = Self(6);
    pub const TRAPDOOR_OPENED_X_NEGATIVE: Self = Self(7);
    // FENCE_GATE
    pub const FENCE_GATE_CLOSED_Z_POSITIVE: Self = Self(0);
    pub const FENCE_GATE_CLOSED_X_POSITIVE: Self = Self(1);
    pub const FENCE_GATE_CLOSED_Z_NEGATIVE: Self = Self(2);
    pub const FENCE_GATE_CLOSED_X_NEGATIVE: Self = Self(3);
    pub const FENCE_GATE_OPENED_Z_POSITIVE: Self = Self(4);
    pub const FENCE_GATE_OPENED_X_POSITIVE: Self = Self(5);
    pub const FENCE_GATE_OPENED_Z_NEGATIVE: Self = Self(6);
    pub const FENCE_GATE_OPENED_X_NEGATIVE: Self = Self(7);
    // QUARTZ
    pub const QUARTZ_NORMAL: Self = Self(0);
    pub const QUARTZ_CHISELLED: Self = Self(1);
    pub const QUARTZ_PILLAR: Self = Self(2);
    // FARMLAND
    pub const FARMLAND_DRY: Self = Self(0);
    pub const FARMLAND_WET: Self = Self(1);
    // LADDER
    pub const LADDER_INVISIBLE_Z_POSITIVE: Self = Self(0);
    pub const LADDER_Z_POSITIVE: Self = Self(2);
    pub const LADDER_Z_NEGATIVE: Self = Self(3);
    pub const LADDER_X_POSITIVE: Self = Self(4);
    pub const LADDER_X_NEGATIVE: Self = Self(5);
    // WALL_SIGN
    pub const WALL_SIGN_Z_POSITIVE: Self = Self(0);
    pub const WALL_SIGN_Z_NEGATIVE: Self = Self(2);
    pub const WALL_SIGN_X_NEGATIVE: Self = Self(4);
    pub const WALL_SIGN_X_POSITIVE: Self = Self(5);
    // NETHER_REACTOR_CORE
    pub const NETHER_REACTOR_CORE_NORMAL: Self = Self(0);
    pub const NETHER_REACTOR_CORE_ACTIVE: Self = Self(1);
    pub const NETHER_REACTOR_CORE_BURNED: Self = Self(2);
    // LEAVES_CARRIED
    pub const LEAVES_CARRIED_DARK_OAK: Self = Self(0);
    pub const LEAVES_CARRIED_DARK_SPRUCE: Self = Self(1);
    pub const LEAVES_CARRIED_DARK_BIRCH: Self = Self(2);
}

/// An entity type supported by the Raspberry Juice/Jam API extensions.
/// These types can be used with [`Command`] to spawn new entities,
/// remove ones of a certain type, get a list of entities of a certain type, and
/// so on.
///
/// See also: [Raspberry Juice Reference Implementation](https://github.com/zhuowei/RaspberryJuice/blob/e8ef1bcd5aa07a1851d25de847c02e0a171d8a20/src/main/resources/mcpi/api/python/modded/mcpi/entity.py#L24-L102)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, FromStr)]
#[repr(transparent)]
pub struct JavaEntityType(pub i32);

impl JavaEntityType {
    /// Used by Raspberry Juice to signify "no filter" in a command that can be
    /// filtered by entity type.
    pub const ANY: i32 = -1;
    pub const EXPERIENCE_ORB: Self = Self(2);
    pub const AREA_EFFECT_CLOUD: Self = Self(3);
    pub const ELDER_GUARDIAN: Self = Self(4);
    pub const WITHER_SKELETON: Self = Self(5);
    pub const STRAY: Self = Self(6);
    pub const EGG: Self = Self(7);
    pub const LEASH_HITCH: Self = Self(8);
    pub const PAINTING: Self = Self(9);
    pub const ARROW: Self = Self(10);
    pub const SNOWBALL: Self = Self(11);
    pub const FIREBALL: Self = Self(12);
    pub const SMALL_FIREBALL: Self = Self(13);
    pub const ENDER_PEARL: Self = Self(14);
    pub const ENDER_SIGNAL: Self = Self(15);
    pub const THROWN_EXP_BOTTLE: Self = Self(17);
    pub const ITEM_FRAME: Self = Self(18);
    pub const WITHER_SKULL: Self = Self(19);
    pub const PRIMED_TNT: Self = Self(20);
    pub const HUSK: Self = Self(23);
    pub const SPECTRAL_ARROW: Self = Self(24);
    pub const SHULKER_BULLET: Self = Self(25);
    pub const DRAGON_FIREBALL: Self = Self(26);
    pub const ZOMBIE_VILLAGER: Self = Self(27);
    pub const SKELETON_HORSE: Self = Self(28);
    pub const ZOMBIE_HORSE: Self = Self(29);
    pub const ARMOR_STAND: Self = Self(30);
    pub const DONKEY: Self = Self(31);
    pub const MULE: Self = Self(32);
    pub const EVOKER_FANGS: Self = Self(33);
    pub const EVOKER: Self = Self(34);
    pub const VEX: Self = Self(35);
    pub const VINDICATOR: Self = Self(36);
    pub const ILLUSIONER: Self = Self(37);
    pub const MINECART_COMMAND: Self = Self(40);
    pub const BOAT: Self = Self(41);
    pub const MINECART: Self = Self(42);
    pub const MINECART_CHEST: Self = Self(43);
    pub const MINECART_FURNACE: Self = Self(44);
    pub const MINECART_TNT: Self = Self(45);
    pub const MINECART_HOPPER: Self = Self(46);
    pub const MINECART_MOB_SPAWNER: Self = Self(47);
    pub const CREEPER: Self = Self(50);
    pub const SKELETON: Self = Self(51);
    pub const SPIDER: Self = Self(52);
    pub const GIANT: Self = Self(53);
    pub const ZOMBIE: Self = Self(54);
    pub const SLIME: Self = Self(55);
    pub const GHAST: Self = Self(56);
    #[doc(alias = "ZOMBIE_PIGMAN")]
    pub const PIG_ZOMBIE: Self = Self(57);
    pub const ENDERMAN: Self = Self(58);
    pub const CAVE_SPIDER: Self = Self(59);
    pub const SILVERFISH: Self = Self(60);
    pub const BLAZE: Self = Self(61);
    pub const MAGMA_CUBE: Self = Self(62);
    pub const ENDER_DRAGON: Self = Self(63);
    pub const WITHER: Self = Self(64);
    pub const BAT: Self = Self(65);
    pub const WITCH: Self = Self(66);
    pub const ENDERMITE: Self = Self(67);
    pub const GUARDIAN: Self = Self(68);
    pub const SHULKER: Self = Self(69);
    pub const PIG: Self = Self(90);
    pub const SHEEP: Self = Self(91);
    pub const COW: Self = Self(92);
    pub const CHICKEN: Self = Self(93);
    pub const SQUID: Self = Self(94);
    pub const WOLF: Self = Self(95);
    pub const MUSHROOM_COW: Self = Self(96);
    pub const SNOWMAN: Self = Self(97);
    pub const OCELOT: Self = Self(98);
    pub const IRON_GOLEM: Self = Self(99);
    pub const HORSE: Self = Self(100);
    pub const RABBIT: Self = Self(101);
    pub const POLAR_BEAR: Self = Self(102);
    pub const LLAMA: Self = Self(103);
    pub const LLAMA_SPIT: Self = Self(104);
    pub const PARROT: Self = Self(105);
    pub const VILLAGER: Self = Self(120);
    pub const ENDER_CRYSTAL: Self = Self(200);
}

/// A key that can be automated (pressed and released) with the MCPI Addons API
/// extension.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display)]
#[as_ref(forward)]
pub struct MCPIExtrasKey<'a>(pub ApiStr<'a>);

impl MCPIExtrasKey<'_> {
    pub const UNKNOWN: Self = Self(ApiStr("UNKNOWN"));

    pub const A: Self = Self(ApiStr("A"));
    pub const D: Self = Self(ApiStr("D"));
    pub const E: Self = Self(ApiStr("E"));
    pub const Q: Self = Self(ApiStr("Q"));
    pub const S: Self = Self(ApiStr("S"));
    pub const T: Self = Self(ApiStr("T"));
    pub const W: Self = Self(ApiStr("W"));
    pub const NUM_1: Self = Self(ApiStr("1"));
    pub const NUM_2: Self = Self(ApiStr("2"));
    pub const NUM_3: Self = Self(ApiStr("3"));
    pub const NUM_4: Self = Self(ApiStr("4"));
    pub const NUM_5: Self = Self(ApiStr("5"));
    pub const NUM_6: Self = Self(ApiStr("6"));
    pub const NUM_7: Self = Self(ApiStr("7"));
    pub const NUM_8: Self = Self(ApiStr("8"));
    pub const NUM_9: Self = Self(ApiStr("9"));
    pub const NUM_0: Self = Self(ApiStr("0"));

    pub const F1: Self = Self(ApiStr("F1"));
    pub const F2: Self = Self(ApiStr("F2"));
    pub const F5: Self = Self(ApiStr("F5"));

    pub const RIGHT: Self = Self(ApiStr("RIGHT"));
    pub const LEFT: Self = Self(ApiStr("LEFT"));
    pub const DOWN: Self = Self(ApiStr("DOWN"));
    pub const UP: Self = Self(ApiStr("UP"));

    pub const LSHIFT: Self = Self(ApiStr("LSHIFT"));
}

/// The color of a sheep.
///
/// This can be when creating a new Sheep entity with
/// [`MCPIExtrasEntityType`].or when changing a sheep's color using
/// [`CustomEntitySetSheepColor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, FromStr)]
pub struct SheepColor(pub i32);

impl SheepColor {
    pub const WHITE: Self = Self(0);
    pub const ORANGE: Self = Self(1);
    pub const MAGENTA: Self = Self(2);
    pub const LIGHT_BLUE: Self = Self(3);
    pub const YELLOW: Self = Self(4);
    pub const LIME: Self = Self(5);
    pub const PINK: Self = Self(6);
    pub const GRAY: Self = Self(7);
    pub const LIGHT_GRAY: Self = Self(8);
    pub const CYAN: Self = Self(9);
    pub const PURPLE: Self = Self(10);
    pub const BLUE: Self = Self(11);
    pub const BROWN: Self = Self(12);
    pub const GREEN: Self = Self(13);
    pub const RED: Self = Self(14);
    pub const BLACK: Self = Self(15);
}

/// An entity type supported by the MCPI Addons API extension.
///
/// See also: [MCPI Addons Reference Implementation](https://github.com/Bigjango13/MCPI-Addons/blob/05027ab7277d51c0dcdd93b58d2ddb66dfea92df/mcpi_addons/entity.py#L56-L100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, FromStr)]
pub struct MCPIExtrasEntityType(pub i32);

impl MCPIExtrasEntityType {
    pub const CHICKEN: Self = Self(10);
    pub const COW: Self = Self(11);
    pub const PIG: Self = Self(12);
    pub const SHEEP: Self = Self(13);
    pub const ZOMBIE: Self = Self(32);
    pub const CREEPER: Self = Self(33);
    pub const SKELETON: Self = Self(34);
    pub const SPIDER: Self = Self(35);
    #[doc(alias = "ZOMBIE_PIGMAN")]
    pub const PIG_ZOMBIE: Self = Self(36);
    #[doc(alias("TILE", "ITEM_ENTITY"))]
    pub const ITEM: Self = Self(64);
    pub const TNT: Self = Self(65);
    #[doc(alias("FALLING_BLOCK"))]
    pub const FALLING_TILE: Self = Self(66);
    pub const ARROW: Self = Self(80);
    pub const SNOWBALL: Self = Self(81);
    pub const EGG: Self = Self(82);
    pub const PAINTING: Self = Self(83);
}

/// An entity type supported by the MCPI Addons API extension.
///
/// This type is primarily used by the [`CustomEntitySpawn`] API call
/// whilst connected to a server with the MCPI Addons API extension.
///
/// The struct itself is an entity type and an entity data value. Entities which
/// do not have a data value are available as associated constants, and ones
/// that do have a data value can be created using one of the provided
/// constructors.
///
/// See also: [MCPI Addons Reference Implementation](https://github.com/Bigjango13/MCPI-Addons/blob/05027ab7277d51c0dcdd93b58d2ddb66dfea92df/mcpi_addons/entity.py#L56-L100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Constructor)]
pub struct MCPIExtrasEntityVariant {
    pub entity: MCPIExtrasEntityType,
    pub value: i32,
}

impl MCPIExtrasEntityVariant {
    pub const CHICKEN: Self = Self::new(MCPIExtrasEntityType::CHICKEN, 0);
    pub const COW: Self = Self::new(MCPIExtrasEntityType::COW, 0);
    pub const PIG: Self = Self::new(MCPIExtrasEntityType::PIG, 0);
    pub const ZOMBIE: Self = Self::new(MCPIExtrasEntityType::ZOMBIE, 0);
    pub const CREEPER: Self = Self::new(MCPIExtrasEntityType::CREEPER, 0);
    pub const SKELETON: Self = Self::new(MCPIExtrasEntityType::SKELETON, 0);
    pub const SPIDER: Self = Self::new(MCPIExtrasEntityType::SPIDER, 0);
    #[doc(alias = "ZOMBIE_PIGMAN")]
    pub const PIG_ZOMBIE: Self = Self::new(MCPIExtrasEntityType::PIG_ZOMBIE, 0);
    pub const SNOWBALL: Self = Self::new(MCPIExtrasEntityType::SNOWBALL, 0);
    pub const EGG: Self = Self::new(MCPIExtrasEntityType::EGG, 0);
    pub const PAINTING: Self = Self::new(MCPIExtrasEntityType::PAINTING, 0);

    /// Creates a new sheep entity type with the given color.
    pub const fn new_sheep(color: SheepColor) -> Self {
        Self {
            entity: MCPIExtrasEntityType::SHEEP,
            value: color.0,
        }
    }

    /// Creates a new dropped item entity type of the given tile.
    pub const fn new_item(tile: Tile) -> Self {
        Self {
            entity: MCPIExtrasEntityType::ITEM,
            value: tile.0 as _,
        }
    }

    /// Creates a new falling tile entity type of the given tile.
    pub const fn new_falling_tile(tile: Tile) -> Self {
        Self {
            entity: MCPIExtrasEntityType::FALLING_TILE,
            value: tile.0 as _,
        }
    }

    /// Creates a new arrow entity that can optionally be a critical hit.
    pub const fn new_arrow(critical: bool) -> Self {
        Self {
            entity: MCPIExtrasEntityType::ARROW,
            value: critical as _,
        }
    }

    /// Creates a new primed TNT entity that will explode after the given number
    /// of ticks.
    pub const fn tnt_from_ticks(ticks: i32) -> Self {
        Self {
            entity: MCPIExtrasEntityType::TNT,
            value: ticks,
        } // TODO: verify time unit
    }

    /// Creates a new primed TNT entity that will explode after the given
    /// duration.
    pub fn new_tnt(fuse: Duration) -> Self {
        Self::tnt_from_ticks((fuse.as_secs_f64() / 0.05) as i32)
    }
}

/// A particle that can be shown using the MCPI Addons API extension.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display)]
#[as_ref(forward)]
pub struct MCPIExtrasParticle<'a>(pub ApiStr<'a>);

impl MCPIExtrasParticle<'_> {
    pub const BUBBLE: Self = Self(ApiStr("bubble"));
    pub const CRIT: Self = Self(ApiStr("crit"));
    pub const FLAME: Self = Self(ApiStr("flame"));
    pub const LAVA: Self = Self(ApiStr("lava"));
    pub const SMOKE: Self = Self(ApiStr("smoke"));
    pub const LARGE_SMOKE: Self = Self(ApiStr("largesmoke"));
    pub const RED_DUST: Self = Self(ApiStr("reddust"));
    pub const IRON_CRACK: Self = Self(ApiStr("ironcrack"));
    pub const SNOWBALL_POOF: Self = Self(ApiStr("snowballpoof"));
    pub const EXPLODE: Self = Self(ApiStr("explode"));
}

/// A particle that can be spawned using the [`WorldSpawnParticle`] API call
/// while connected to a server with the Raspberry Jam API extension.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display)]
#[as_ref(forward)]
pub struct RaspberryJamParticle<'a>(pub ApiStr<'a>);

impl RaspberryJamParticle<'_> {
    pub const BARRIER: Self = Self(ApiStr("BARRIER"));
    pub const BLOCK_CRACK: Self = Self(ApiStr("BLOCK_CRACK"));
    pub const BLOCK_DUST: Self = Self(ApiStr("BLOCK_DUST"));
    pub const CLOUD: Self = Self(ApiStr("CLOUD"));
    pub const CRIT: Self = Self(ApiStr("CRIT"));
    pub const CRIT_MAGIC: Self = Self(ApiStr("CRIT_MAGIC"));
    pub const DRIP_LAVA: Self = Self(ApiStr("DRIP_LAVA"));
    pub const DRIP_WATER: Self = Self(ApiStr("DRIP_WATER"));
    pub const ENCHANTMENT_TABLE: Self = Self(ApiStr("ENCHANTMENT_TABLE"));
    pub const EXPLOSION_HUGE: Self = Self(ApiStr("EXPLOSION_HUGE"));
    pub const EXPLOSION_LARGE: Self = Self(ApiStr("EXPLOSION_LARGE"));
    pub const EXPLOSION_NORMAL: Self = Self(ApiStr("EXPLOSION_NORMAL"));
    pub const FIREWORKS_SPARK: Self = Self(ApiStr("FIREWORKS_SPARK"));
    pub const FLAME: Self = Self(ApiStr("FLAME"));
    pub const FOOTSTEP: Self = Self(ApiStr("FOOTSTEP"));
    pub const HEART: Self = Self(ApiStr("HEART"));
    pub const ITEM_CRACK: Self = Self(ApiStr("ITEM_CRACK"));
    pub const ITEM_TAKE: Self = Self(ApiStr("ITEM_TAKE"));
    pub const LAVA: Self = Self(ApiStr("LAVA"));
    pub const MOB_APPEARANCE: Self = Self(ApiStr("MOB_APPEARANCE"));
    pub const NOTE: Self = Self(ApiStr("NOTE"));
    pub const PORTAL: Self = Self(ApiStr("PORTAL"));
    pub const REDSTONE: Self = Self(ApiStr("REDSTONE"));
    pub const SLIME: Self = Self(ApiStr("SLIME"));
    pub const SMOKE_LARGE: Self = Self(ApiStr("SMOKE_LARGE"));
    pub const SMOKE_NORMAL: Self = Self(ApiStr("SMOKE_NORMAL"));
    pub const SNOW_SHOVEL: Self = Self(ApiStr("SNOW_SHOVEL"));
    pub const SNOWBALL: Self = Self(ApiStr("SNOWBALL"));
    pub const SPELL: Self = Self(ApiStr("SPELL"));
    pub const SPELL_INSTANT: Self = Self(ApiStr("SPELL_INSTANT"));
    pub const SPELL_MOB: Self = Self(ApiStr("SPELL_MOB"));
    pub const SPELL_MOB_AMBIENT: Self = Self(ApiStr("SPELL_MOB_AMBIENT"));
    pub const SPELL_WITCH: Self = Self(ApiStr("SPELL_WITCH"));
    pub const SUSPENDED: Self = Self(ApiStr("SUSPENDED"));
    pub const SUSPENDED_DEPTH: Self = Self(ApiStr("SUSPENDED_DEPTH"));
    pub const TOWN_AURA: Self = Self(ApiStr("TOWN_AURA"));
    pub const VILLAGER_ANGRY: Self = Self(ApiStr("VILLAGER_ANGRY"));
    pub const VILLAGER_HAPPY: Self = Self(ApiStr("VILLAGER_HAPPY"));
    pub const WATER_BUBBLE: Self = Self(ApiStr("WATER_BUBBLE"));
    pub const WATER_DROP: Self = Self(ApiStr("WATER_DROP"));
    pub const WATER_SPLASH: Self = Self(ApiStr("WATER_SPLASH"));
    pub const WATER_WAKE: Self = Self(ApiStr("WATER_WAKE"));
}

/// A world dimension that can be used with the Raspberry Jam API extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, FromStr)]
pub struct Dimension(pub i32);

impl Dimension {
    pub const OVERWORLD: Self = Self(0);
    pub const NETHER: Self = Self(-1);
    pub const END: Self = Self(1);
}

/// A player-related setting that can be updated using the API.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display)]
#[as_ref(forward)]
pub struct PlayerSettingKey<'a>(pub ApiStr<'a>);

impl PlayerSettingKey<'_> {
    /// When enabled, the player will automatically jump when walking into a
    /// block.
    pub const AUTOJUMP: Self = Self(ApiStr("autojump"));
}

/// A world-related setting that can be updated using the API.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display)]
#[as_ref(forward)]
pub struct WorldSettingKey<'a>(pub ApiStr<'a>);

impl WorldSettingKey<'_> {
    /// When enabled, players cannot edit the world (such as by placing or
    /// destroying blocks).
    pub const WORLD_IMMUTABLE: Self = Self(ApiStr("world_immutable"));
    /// When disabled, player name tags will not be shown above their heads.
    pub const NAMETAGS_VISIBLE: Self = Self(ApiStr("nametags_visible"));
    /// Raspberry Jam extension: controls whether NBT data will be included when
    /// fetching block data.
    pub const INCLUDE_NBT_WITH_DATA: Self = Self(ApiStr("include_nbt_with_data"));
    /// Raspberry Jam extension: while enabled, block updates requested over the
    /// API will be queued but not executed.
    pub const PAUSE_DRAWING: Self = Self(ApiStr("pause_drawing"));
}

/// An event-related setting that can be updated using the API. (Raspberry Jam
/// extension)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef)]
#[as_ref(forward)]
pub struct EventsSettingKey<'a>(pub ApiStr<'a>);

impl WorldSettingKey<'_> {
    /// Raspberry Jam extension: controls whether events will only be sent from
    /// players holding a sword.
    pub const RESTRICT_TO_SWORD: Self = Self(ApiStr("restrict_to_sword"));
    /// Raspberry Jam extension: controls whether events will be sent that were
    /// triggered by left-clicks.
    pub const DETECT_LEFT_CLICK: Self = Self(ApiStr("detect_left_click"));
}

/// The identifier of an entity in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, FromStr)]
pub struct EntityId(pub i32);

// MARK: Commands

pub mod commands;

/// A string that does not contain the LF (line feed) character.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display)]
pub struct ApiStr<'a>(pub &'a str);

impl<'a> ApiStr<'a> {
    /// Creates a new ApiString from the given string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains a LF (line feed) character.
    pub fn new(inner: &'a str) -> Result<Self, NewlineStrError> {
        if inner.contains('\n') {
            NewlineStrSnafu.fail()
        } else {
            Ok(Self(inner))
        }
    }

    /// Creates a new ApiString from the given string without checking for LF
    /// characters.
    ///
    /// # Safety
    ///
    /// The string must not contain LF (line feed) characters.
    #[must_use]
    pub const unsafe fn new_unchecked(inner: &'a str) -> Self {
        Self(inner)
    }
}

impl<'a> TryFrom<&'a str> for ApiStr<'a> {
    type Error = NewlineStrError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// An error that occurs when an [`ApiStr`] is created that contains a LF (line
/// feed) character.
#[derive(Debug, Snafu)]
#[snafu(display("String must not contain LF characters."))]
pub struct NewlineStrError;

#[derive(Debug, Snafu)]
pub enum ChatStringError {
    #[snafu(display("{source}"), context(false))]
    Newline {
        source: NewlineStrError,
    },
    CP437,
}

/// A CP437 string that does not contain the LF (line feed) character.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef)]
#[as_ref(forward)]
pub struct ChatString<'a>(Cp437String<'a>);

impl FromStr for ChatString<'_> {
    type Err = ChatStringError;

    /// Creates a new [`ChatString`] from the given string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains a LF (line feed) character.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('\n') {
            Err(NewlineStrSnafu.build().into())
        } else {
            let cp437 = Cp437String::from_utf8(s).context(CP437Snafu)?;
            Ok(Self(cp437))
        }
    }
}

impl ChatString<'_> {
    /// Creates a new [`ChatString`] from the given string.
    ///
    /// Invalid characters are replaced with the "?" character.
    #[must_use]
    pub fn from_str_lossy(inner: &str) -> Self {
        let replacement = CHAR_TO_CP437[&'?'];
        let converted_bytes = inner
            .chars()
            .map(|c| if c == '\n' { '?' } else { c })
            .map(|c| CHAR_TO_CP437.get(&c).cloned().unwrap_or(replacement))
            .collect();
        Self(Cp437String(Cow::Owned(converted_bytes)))
    }

    /// Creates a new [`ChatString`] from the given [`Cp437String`].
    ///
    /// # Safety
    ///
    /// The string must be CP437-encoded and not contain LF (line feed)
    /// characters.
    #[must_use]
    pub const unsafe fn new_unchecked(inner: Cp437String<'static>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn to_utf8(&self) -> String {
        self.0.to_string()
    }
}

// MARK: Connection

/// An error that can occur when interacting with a Minecraft: Pi Edition game
/// server.
#[derive(Debug, Snafu)]
pub enum ConnectionError {
    /// An IO error occurred and the command could not be sent.
    #[snafu(display("The command could not be sent: {source}"), context(false))]
    Io {
        source: std::io::Error,
        backtrace: Backtrace,
    },
    /// The server responded with a 'Fail' message.
    ///
    /// This usually means that the server could not parse the command.
    #[snafu(display("The server responded with a 'Fail' message."))]
    GenericFail { backtrace: Backtrace },
    /// The server did not respond in time.
    ///
    /// This error will only be triggered for commands that require a response.
    #[snafu(display("The server did not respond in {timeout:?}."))]
    NoResponse {
        timeout: Duration,
        backtrace: Backtrace,
    },
    /// Failed to parse a server response as UTF-8.
    #[snafu(
        display("Failed to parse server response as UTF-8: {source}"),
        context(false)
    )]
    ResponseNotUtf8 {
        source: std::string::FromUtf8Error,
        backtrace: Backtrace,
    },
    /// The server unexpectedly closed the connection.
    ConnectionClosed { backtrace: Backtrace },
    /// The server did not respond in time.
    #[snafu(
        display("The server did not respond in time: {source}"),
        context(false)
    )]
    Timeout {
        source: Elapsed,
        backtrace: Backtrace,
    },
    /// Failed to queue request: channel closed.
    Send { backtrace: Backtrace },
    /// Request failed.
    #[snafu(display("Request failed: {source}"), context(false))]
    Recv {
        source: RecvError,
        backtrace: Backtrace,
    },
    /// Request queue full.
    QueueFull { backtrace: Backtrace },
}

/// Options that can be set to change the behavior of the connection to the
/// game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectOptions {
    /// The amount of time to wait for a response from the server before giving
    /// up. Setting this to a higher value may slow performance,
    /// but has a smaller chance of causing a timeout error.
    ///
    /// Defaults to 1 second.
    pub response_timeout: Option<Duration>,
    /// Whether to always wait for a response from the server.
    ///
    /// Because the server (under normal circumstances) does not acknowledge
    /// commands that do not require a response, the default behavior is to
    /// not check for a response for those commands. However, in an error
    /// scenario, the server may respond with a 'Fail' message, which would be
    /// missed if this setting is disabled.
    ///
    /// Enabling this setting will significantly degrade performance because
    /// commands that do not require a response will need to wait
    /// [`response_timeout`] seconds before continuing.
    pub always_wait_for_response: bool,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            response_timeout: Some(Duration::from_secs(1)),
            always_wait_for_response: false,
        }
    }
}

/// A communication interface with a Minecraft: Pi Edition game server.
pub trait Protocol: Debug {
    /// Sends a command to the server and returns its response without
    /// processing or parsing.
    fn send<T: SerializableCommand>(
        &mut self,
        command: T,
    ) -> impl Future<Output = Result<String, ConnectionError>> + Send;

    /// Flushes the connection and disconnects.
    fn close(&mut self) -> impl Future<Output = Result<(), ConnectionError>> + Send;
}

/// A connection to a game server using the Minecraft: Pi Edition API protocol.
#[derive(Debug)]
pub struct ServerConnection {
    socket: BufWriter<TcpStream>,
    buffer: String,
    pub options: ConnectOptions,
}

impl From<BufWriter<TcpStream>> for ServerConnection {
    fn from(value: BufWriter<TcpStream>) -> Self {
        Self {
            socket: value,
            buffer: String::new(),
            options: ConnectOptions::default(),
        }
    }
}

impl ServerConnection {
    /// Connects to the Minecraft: Pi Edition server at the given address.
    pub async fn new(addr: impl ToSocketAddrs, options: ConnectOptions) -> std::io::Result<Self> {
        let socket = TcpStream::connect(addr).await?;
        Ok(Self {
            socket: BufWriter::new(socket),
            buffer: String::new(),
            options,
        })
    }

    /// Creates a [`ServerConnection`] from an existing TCP steam.
    pub fn from_stream(socket: TcpStream, options: ConnectOptions) -> Self {
        Self {
            socket: BufWriter::new(socket),
            buffer: String::new(),
            options,
        }
    }

    /// Sends a raw command to the server.
    ///
    /// # Panics
    ///
    /// The function will panic if the
    /// [`ConnectOptions::always_wait_for_response`] option is set, there is
    /// no [`ConnectOptions::response_timeout`], and the command being sent does
    /// not [expect a response](`SerializableCommand::HAS_RESPONSE`) in order to
    /// prevent an infinite hang.
    pub(crate) async fn send_raw(
        &mut self,
        data: &[u8],
        has_response: bool,
    ) -> Result<String, ConnectionError> {
        self.socket.write_all(data).await?;
        self.socket.flush().await?;

        if has_response || self.options.always_wait_for_response {
            if let Some(response_timeout) = self.options.response_timeout {
                timeout(response_timeout, self.read_frame()).await?
            } else {
                if has_response {
                    panic!("Using the `always_wait_for_response` setting without a `response_timeout` for a command that does not expect a response may cause an infinite hang.");
                }
                self.read_frame().await
            }
        } else {
            Ok(String::new())
        }
    }

    /// Receive a frame from the connection by either using data that has
    /// already been received or waiting for more data from the socket.
    pub(crate) async fn read_frame(&mut self) -> Result<String, ConnectionError> {
        loop {
            // Attempt to parse a frame from the buffered data. If enough data
            // has already been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame() {
                return Ok(frame);
            }

            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            let bytes_read = self.socket.read_to_string(&mut self.buffer).await?;
            if bytes_read == 0 {
                // Connection lost.
                return ConnectionClosedSnafu.fail();
            }
        }
    }

    /// Attempt to parse a frame from data that has already been received.
    pub(crate) fn parse_frame(&mut self) -> Option<String> {
        let idx = self.buffer.find('\n')?;
        let frame = self.buffer.drain(..idx + 1).collect();
        Some(frame)
    }
}

impl Protocol for ServerConnection {
    /// Sends a command to the server and returns its response.
    ///
    /// If the command does not [expect a
    /// response](`SerializableCommand::HAS_RESPONSE`)
    /// and the [`ConnectOptions::always_wait_for_response`] option has not been
    /// changed to `true`, an empty string is returned without waiting for
    /// the server to respond.
    ///
    /// The operation will time out after the duration specified in the
    /// [`ConnectOptions::response_timeout`] option.
    ///
    /// Server responses are returned without processing or parsing.
    ///
    /// # Panics
    ///
    /// The function will panic if the
    /// [`ConnectOptions::always_wait_for_response`] option is set, there is
    /// no [`ConnectOptions::response_timeout`], and the command being sent does
    /// not [expect a response](`SerializableCommand::HAS_RESPONSE`) in order to
    /// prevent an infinite hang.
    async fn send<T: SerializableCommand>(
        &mut self,
        command: T,
    ) -> Result<String, ConnectionError> {
        self.send_raw(&command.to_command_bytes(), T::HAS_RESPONSE)
            .await
    }

    async fn close(&mut self) -> Result<(), ConnectionError> {
        self.socket.shutdown().await?;
        Ok(())
    }
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use commands::ChatPost;

    use super::*;

    #[test]
    fn api_str_and_chat_string_allow_special_characters() {
        let string = ChatString::from_str_lossy("I am so happy ♥");
        let command = ChatPost { message: string };
        assert_eq!(
            command.to_command_bytes(),
            b"chat.post(I am so happy \x03)\n"
        );
        let string = ApiStr::new("I am so happy ♥").unwrap();
        assert_eq!(string.to_string(), "I am so happy ♥");
    }

    #[test]
    fn mcpi_extras_entity_new_sheep() {
        let entity = MCPIExtrasEntityVariant::new_sheep(SheepColor(1));
        assert_eq!(
            entity,
            MCPIExtrasEntityVariant {
                entity: MCPIExtrasEntityType::SHEEP,
                value: 1
            }
        );
    }

    #[test]
    fn mcpi_extras_entity_new_arrow() {
        let entity = MCPIExtrasEntityVariant::new_arrow(true);
        assert_eq!(
            entity,
            MCPIExtrasEntityVariant {
                entity: MCPIExtrasEntityType::ARROW,
                value: 1
            }
        );
    }

    #[test]
    fn mcpi_extras_entity_new_dropped_block() {
        let entity = MCPIExtrasEntityVariant::new_item(Tile(1));
        assert_eq!(
            entity,
            MCPIExtrasEntityVariant {
                entity: MCPIExtrasEntityType::ITEM,
                value: 1
            }
        );
    }

    #[test]
    fn mcpi_extras_entity_new_falling_block() {
        let entity = MCPIExtrasEntityVariant::new_falling_tile(Tile(1));
        assert_eq!(
            entity,
            MCPIExtrasEntityVariant {
                entity: MCPIExtrasEntityType::FALLING_TILE,
                value: 1
            }
        );
    }

    #[test]
    fn mcpi_extras_entity_new_tnt_duration() {
        let entity = MCPIExtrasEntityVariant::new_tnt(Duration::from_secs(1));
        assert_eq!(
            entity,
            MCPIExtrasEntityVariant {
                entity: MCPIExtrasEntityType::TNT,
                value: 20
            }
        );
    }

    #[test]
    fn mcpi_extras_entity_new_tnt_ticks() {
        let entity = MCPIExtrasEntityVariant::tnt_from_ticks(1);
        assert_eq!(
            entity,
            MCPIExtrasEntityVariant {
                entity: MCPIExtrasEntityType::TNT,
                value: 1
            }
        );
    }
}
