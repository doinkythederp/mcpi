//! Implementation of the Minecraft: Pi Edition protocol.
//!
//! Sources include:
//! - [Picraft docs](https://picraft.readthedocs.io/en/release-1.0/protocol.html)
//! - [Wiki.vg](https://wiki.vg/Minecraft_Pi_Protocol)
//! - [martinohanlon/Minecraft-Pi-API](https://github.com/martinohanlon/Minecraft-Pi-API/blob/master/api.md)
//! - [MCPI Revival Wiki](https://mcpirevival.miraheze.org/wiki/MCPI_Revival)
//!

use std::fmt::{self, Debug, Display, Formatter};
use std::future::Future;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use nalgebra::{Point, Point2, Point3, Scalar};
use snafu::{Backtrace, OptionExt, Snafu};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::oneshot::error::RecvError;
use tokio::time::error::Elapsed;
use tokio::time::timeout;

use crate::util::{cp437_to_string, str_to_cp437, str_to_cp437_lossy};

pub mod queued;

// MARK: Enums

/// A block that can be used in Minecraft: Pi Edition.
///
/// Vanilla blocks are available as associated constants.
///
/// See also: [Minecraft: Pi Edition Complete Block List](https://mcpirevival.miraheze.org/wiki/Minecraft:_Pi_Edition_Complete_Block_List)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
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
    /// This block is a darker version of [`LEAVES`].
    pub const LEAVES_CARRIED: Self = Self(254);
    /// This block is a duplicate of [`STONE`] with a different ID.
    pub const STONE_1: Self = Self(255);

    /// Returns a helper struct that can be converted to a human-readable version of the block name.
    pub const fn display(self) -> TileDisplay {
        TileDisplay::new(self)
    }
}

impl Deref for Tile {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// Helper struct for converting a [`Tile`] to a human-readable string.
///
/// Implements a human-readable [`Display`] for [`Tile`].
pub struct TileDisplay(Tile);

impl TileDisplay {
    pub const fn new(tile: Tile) -> Self {
        Self(tile)
    }
}

impl Display for TileDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
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
/// For many blocks, this data is used to represent the block's state, such as growth stage or orientation.
/// These common values are available as associated constants.
///
/// When working with blocks that don't store any extra state, the TileData will not be used by the server, but can be
/// set and later retrieved by the API user.
///
/// See also: [Minecraft: Pi Edition Complete Block List](https://mcpirevival.miraheze.org/wiki/Minecraft:_Pi_Edition_Complete_Block_List)
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TileData(pub u8);

impl TileData {
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

impl Deref for TileData {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for TileData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// An entity type supported by the Raspberry Juice/Jam API extensions.
/// These types can be used with [`Command`] to spawn new entities,
/// remove ones of a certain type, get a list of entities of a certain type, and so on.
///
/// See also: [Raspberry Juice Reference Implementation](https://github.com/zhuowei/RaspberryJuice/blob/e8ef1bcd5aa07a1851d25de847c02e0a171d8a20/src/main/resources/mcpi/api/python/modded/mcpi/entity.py#L24-L102)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct JavaEntityType(pub i32);

impl JavaEntityType {
    /// Used by Raspberry Juice to signify "no filter" in a command that can be filtered by entity type.
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

impl Deref for JavaEntityType {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for JavaEntityType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A key that can be automated (pressed and released) with the MCPI Addons API extension.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Display for MCPIExtrasKey<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<'a> Deref for MCPIExtrasKey<'a> {
    type Target = ApiStr<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The color of a sheep.
///
/// This can be when creating a new Sheep entity with [`MCPIExtrasEntityType`].or when
/// changing a sheep's color using [`Command::CustomEntitySetSheepColor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Deref for SheepColor {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for SheepColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// An entity type supported by the MCPI Addons API extension.
///
/// This type is primarily used by the [`Command::CustomEntitySpawn`] API call
/// while connected to a server with the MCPI Addons API extension.
///
/// The struct itself is an entity type and an entity data value. Entities which do not have a data value
/// are available as associated constants, and ones that do have a data value can be created using one
/// of the provided constructors.
///
/// See also: [MCPI Addons Reference Implementation](https://github.com/Bigjango13/MCPI-Addons/blob/05027ab7277d51c0dcdd93b58d2ddb66dfea92df/mcpi_addons/entity.py#L56-L100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MCPIExtrasEntityType(pub i32, pub i32);

impl MCPIExtrasEntityType {
    pub const CHICKEN: Self = Self(10, 0);
    pub const COW: Self = Self(11, 0);
    pub const PIG: Self = Self(12, 0);
    pub const SHEEP: i32 = 13;
    pub const ZOMBIE: Self = Self(32, 0);
    pub const CREEPER: Self = Self(33, 0);
    pub const SKELETON: Self = Self(34, 0);
    pub const SPIDER: Self = Self(35, 0);
    #[doc(alias = "ZOMBIE_PIGMAN")]
    pub const PIG_ZOMBIE: Self = Self(36, 0);
    #[doc(alias("TILE", "ITEM_ENTITY"))]
    pub const ITEM: i32 = 64;
    pub const TNT: i32 = 65;
    #[doc(alias("FALLING_BLOCK"))]
    pub const FALLING_TILE: i32 = 66;
    pub const ARROW: i32 = 80;
    pub const SNOWBALL: Self = Self(81, 0);
    pub const EGG: Self = Self(82, 0);
    pub const PAINTING: Self = Self(83, 0);

    /// Creates a new sheep entity type with the given color.
    pub const fn new_sheep(color: SheepColor) -> Self {
        Self(Self::SHEEP, color.0)
    }

    /// Creates a new dropped item entity type of the given tile.
    pub const fn new_item(tile: Tile) -> Self {
        Self(Self::ITEM, tile.0 as _)
    }

    /// Creates a new falling tile entity type of the given tile.
    pub const fn new_falling_tile(tile: Tile) -> Self {
        Self(Self::FALLING_TILE, tile.0 as _)
    }

    /// Creates a new arrow entity that can optionally be a critical hit.
    pub const fn new_arrow(critical: bool) -> Self {
        Self(Self::ARROW, critical as _)
    }

    /// Creates a new primed TNT entity that will explode after the given number of ticks.
    pub const fn new_tnt_const(ticks: i32) -> Self {
        Self(Self::TNT, ticks) // TODO: what is this acually measured in? i've never checked
    }

    /// Creates a new primed TNT entity that will explode after the given duration.
    pub fn new_tnt(fuse: Duration) -> Self {
        Self::new_tnt_const((fuse.as_secs_f64() / 0.05) as i32)
    }
}

/// A particle that can be shown using the MCPI Addons API extension.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl<'a> Deref for MCPIExtrasParticle<'a> {
    type Target = ApiStr<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Display for MCPIExtrasParticle<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A particle that can be spawned using the [`Command::WorldSpawnParticle`] API call
/// while connected to a server with the Raspberry Jam API extension.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl<'a> Deref for RaspberryJamParticle<'a> {
    type Target = ApiStr<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Display for RaspberryJamParticle<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A world dimension that can be used with the Raspberry Jam API extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dimension(pub i32);

impl Dimension {
    pub const OVERWORLD: Self = Self(0);
    pub const NETHER: Self = Self(-1);
    pub const END: Self = Self(1);
}

impl Deref for Dimension {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A player-related setting that can be updated using the API.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerSettingKey<'a>(pub ApiStr<'a>);

impl PlayerSettingKey<'_> {
    /// When enabled, the player will automatically jump when walking into a block.
    pub const AUTOJUMP: Self = Self(ApiStr("autojump"));
}

impl<'a> Deref for PlayerSettingKey<'a> {
    type Target = ApiStr<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Display for PlayerSettingKey<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A world-related setting that can be updated using the API.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldSettingKey<'a>(pub ApiStr<'a>);

impl WorldSettingKey<'_> {
    /// When enabled, players cannot edit the world (such as by placing or destroying blocks).
    pub const WORLD_IMMUTABLE: Self = Self(ApiStr("world_immutable"));
    /// When disabled, player name tags will not be shown above their heads.
    pub const NAMETAGS_VISIBLE: Self = Self(ApiStr("nametags_visible"));
    /// Raspberry Jam extension: controls whether NBT data will be included when fetching block data.
    pub const INCLUDE_NBT_WITH_DATA: Self = Self(ApiStr("include_nbt_with_data"));
    /// Raspberry Jam extension: while enabled, block updates requested over the API will be queued but not executed.
    pub const PAUSE_DRAWING: Self = Self(ApiStr("pause_drawing"));
}

impl<'a> Deref for WorldSettingKey<'a> {
    type Target = ApiStr<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Display for WorldSettingKey<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// An event-related setting that can be updated using the API. (Raspberry Jam extension)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventsSettingKey<'a>(pub ApiStr<'a>);

impl WorldSettingKey<'_> {
    /// Raspberry Jam extension: controls whether events will only be sent from players holding a sword.
    pub const RESTRICT_TO_SWORD: Self = Self(ApiStr("restrict_to_sword"));
    /// Raspberry Jam extension: controls whether events will be sent that were triggered by left-clicks.
    pub const DETECT_LEFT_CLICK: Self = Self(ApiStr("detect_left_click"));
}

impl<'a> Deref for EventsSettingKey<'a> {
    type Target = ApiStr<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Display for EventsSettingKey<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// The identifier of an entity in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub i32);

impl Deref for EntityId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

// MARK: Commands

/// A command that can be sent to the game server to perform an action or query information.
///
/// Includes all commands supported by the vanilla Minecraft: Pi Edition game, as well as
/// commands from the following plugins, mods, or API extensions:
///
/// - [Raspberry Juice](https://dev.bukkit.org/projects/raspberryjuice) plugin
/// - [MCPI Addons](https://github.com/Bigjango13/MCPI-Addons) mod
/// - [Raspberry Jam](https://github.com/arpruss/raspberryjammod)
///
/// Enum members are generally named after the API method they correspond to, with the exception of
/// a few extension commands that have conflicting names.
#[derive(Debug)]
#[non_exhaustive]
pub enum Command<'a> {
    // # Vanilla Commands

    // Camera APIs
    CameraModeSetFixed,
    CameraModeSetFollow {
        target: Option<EntityId>,
    },
    CameraModeSetNormal {
        target: Option<EntityId>,
    },
    // TODO: Test whether this works on vanilla
    CameraModeSetThirdPerson {
        target: Option<EntityId>,
    },
    CameraSetPos(Point3<f64>),
    // Chat APIs
    ChatPost(&'a ChatString),
    // Entity APIs
    EntityGetPos(EntityId),
    EntityGetTile(EntityId),
    EntitySetPos(EntityId, Point3<f64>),
    EntitySetTile(EntityId, Point3<i16>),
    // Player APIs
    PlayerGetPos,
    PlayerGetTile,
    PlayerSetPos(Point3<f64>),
    PlayerSetTile(Point3<i16>),
    PlayerSetting {
        key: PlayerSettingKey<'a>,
        value: bool,
    },
    // World APIs
    WorldCheckpointRestore,
    WorldCheckpointSave,
    WorldGetBlock(Point3<i16>),
    /// Has Raspberry Jam mod extension to get block with NBT data. Requires a world setting to be set.
    /// TODO: look into this
    WorldGetBlockWithData(Point3<i16>),
    WorldGetHeight(Point2<i16>),
    WorldGetPlayerIds,
    WorldSetBlock {
        coords: Point3<i16>,
        tile: Tile,
        data: TileData,
        /// Raspberry Jam mod extension to set block with NBT data.
        ///
        /// Set to [`None`] when using other servers.
        json_nbt: Option<ApiStr<'a>>,
    },
    WorldSetBlocks {
        coords_1: Point3<i16>,
        coords_2: Point3<i16>,
        tile: Tile,
        data: TileData,
        /// Raspberry Jam mod extension to add NBT data to the blocks being set.
        ///
        /// Set to [`None`] when using other servers.
        json_nbt: Option<ApiStr<'a>>,
    },
    WorldSetting {
        key: WorldSettingKey<'a>,
        value: bool,
    },
    // Event APIs
    EventsClear,
    EventsBlockHits,

    // # Raspberry Juice (& Raspberry Jam) Extensions
    // https://dev.bukkit.org/projects/raspberryjuice

    // World APIs
    WorldGetBlocks(Point3<i16>, Point3<i16>),
    /// When using the Raspberry Jam mod, this can be set to [`None`] to get the current player's ID.
    WorldGetPlayerId(Option<ApiStr<'a>>),
    WorldGetEntities(Option<JavaEntityType>),
    WorldRemoveEntity(EntityId),
    WorldRemoveEntities(Option<JavaEntityType>),
    WorldSetSign {
        coords: Point3<i16>,
        tile: Tile,
        data: TileData,
        lines: Vec<ApiStr<'a>>,
    },
    RaspberryJuiceWorldSpawnEntity {
        coords: Point3<f64>,
        entity_type: JavaEntityType,
    },
    WorldGetEntityTypes,

    // Entity APIs
    EntityGetName(EntityId),
    EntityGetDirection(EntityId),
    EntitySetDirection(EntityId, Point3<f64>),
    EntityGetPitch(EntityId),
    EntitySetPitch(EntityId, f32),
    EntityGetRotation(EntityId),
    EntitySetRotation(EntityId, f32),
    EntityEventsClear(EntityId),
    EntityEventsBlockHits(EntityId),
    EntityEventsChatPosts(EntityId),
    EntityEventsProjectileHits(EntityId),
    EntityGetEntities {
        target: EntityId,
        distance: i32,
        entity_type: Option<JavaEntityType>,
    },
    EntityRemoveEntities {
        target: EntityId,
        distance: i32,
        entity_type: Option<JavaEntityType>,
    },

    // Player APIs
    PlayerGetAbsPos,
    PlayerSetAbsPos(Point3<f64>),
    PlayerSetDirection(Point3<f64>),
    PlayerGetDirection,
    PlayerSetRotation(f32),
    PlayerGetRotation,
    PlayerSetPitch(f32),
    PlayerGetPitch,
    PlayerEventsClear,
    PlayerEventsBlockHits,
    PlayerEventsChatPosts,
    PlayerEventsProjectileHits,
    PlayerGetEntities {
        distance: i32,
        entity_type: Option<JavaEntityType>,
    },
    PlayerRemoveEntities {
        distance: i32,
        entity_type: Option<JavaEntityType>,
    },

    // Events APIs
    EventsChatPosts,
    EventsProjectileHits,

    // # MCPI Addons mod by Bigjango13
    // https://github.com/Bigjango13/MCPI-Addons

    // Custom Log APIs
    CustomLogDebug(ApiStr<'a>),
    CustomLogInfo(ApiStr<'a>),
    CustomLogWarn(ApiStr<'a>),
    CustomLogErr(ApiStr<'a>),

    // Custom Inventory APIs
    CustomInventoryGetSlot,
    CustomInventoryUnsafeGive {
        id: Option<i32>,
        auxillary: Option<i32>,
        count: Option<i32>,
    },
    CustomInventoryGive {
        id: Option<i32>,
        auxillary: Option<i32>,
        count: Option<i32>,
    },

    // Custom Override APIs
    CustomOverrideReset,
    CustomOverride {
        before: Tile,
        after: Tile,
    },

    // Custom Post APIs
    CustomPostClient(ApiStr<'a>),
    CustomPostNoPrefix(ApiStr<'a>),

    // Custom Key APIs
    CustomKeyPress(MCPIExtrasKey<'a>),
    CustomKeyRelease(MCPIExtrasKey<'a>),

    // Custom Username APIs
    CustomUsernameAll,

    // Custom World API
    CustomWorldParticle {
        particle: MCPIExtrasParticle<'a>,
        coords: Point3<f32>,
    },
    CustomWorldDir,
    CustomWorldName,
    CustomWorldServername,

    // Custom Player APIs
    CustomPlayerGetHealth,
    CustomPlayerSetHealth(i32),
    CustomPlayerCloseGUI,
    CustomPlayerGetGamemode,

    // Custom Entity APIs
    CustomEntitySpawn {
        entity_type: MCPIExtrasEntityType,
        health: i32,
        coords: Point3<f32>,
        direction: Point2<f32>, // TODO: is this the most correct type?
    },
    CustomEntitySetAge {
        entity_id: EntityId,
        age: i32,
    },
    CustomEntitySetSheepColor {
        entity_id: EntityId,
        color: SheepColor,
    },

    // Chat Events APIs
    EventsChatSize,

    // Custom Reborn APIs
    CustomRebornVersion,
    CustomRebornFeature(ApiStr<'a>),

    // Entity APIs
    EntityGetAllEntities,

    // # Raspbery Jam mod
    // https://github.com/arpruss/raspberryjammod

    // World APIs
    /// Has extension to get block with NBT data. Requires a world setting to be set.
    WorldGetBlocksWithData {
        coords_1: Point3<i16>,
        coords_2: Point3<i16>,
    },
    WorldSpawnParticle {
        particle: RaspberryJamParticle<'a>,
        coords: Point3<f64>,
        direction: Point3<f64>, // TODO: Unclear how to use this
        speed: f64,
        count: i32,
    },

    // Block APIs
    BlockGetLightLevel {
        tile: Tile,
    },
    BlockSetLightLevel {
        tile: Tile,
        level: f32,
    },

    // Entity APIs
    EntitySetDimension {
        entity_id: EntityId,
        dimension: Dimension,
    },
    EntityGetNameAndUUID(EntityId),
    RaspberryJamWorldSpawnEntity {
        entity_type: JavaEntityType,
        coords: Point3<f64>,
        json_nbt: Option<ApiStr<'a>>,
    },

    // Player APIs
    PlayerSetDimension {
        dimension: Dimension,
    },
    PlayerGetNameAndUUID,

    // Camera APIs
    CameraGetEntityId,
    CameraSetFollow {
        target: Option<EntityId>,
    },
    CameraSetNormal {
        target: Option<EntityId>,
    },
    CameraSetThirdPerson {
        target: Option<EntityId>,
    },
    CameraSetDebug,
    CameraSetDistance(f32),
}

impl Command<'_> {
    #[must_use]
    pub const fn has_response(&self) -> bool {
        match self {
            Self::CameraModeSetFixed
            | Self::CameraModeSetFollow { .. }
            | Self::CameraModeSetNormal { .. }
            | Self::CameraModeSetThirdPerson { .. }
            | Self::CameraSetPos(_)
            | Self::ChatPost(_)
            | Self::EntitySetPos(_, _)
            | Self::EntitySetTile(_, _)
            | Self::PlayerSetPos(_)
            | Self::PlayerSetTile(_)
            | Self::PlayerSetting { .. }
            | Self::WorldCheckpointRestore
            | Self::WorldCheckpointSave
            | Self::WorldSetBlock { .. }
            | Self::WorldSetBlocks { .. }
            | Self::WorldSetting { .. }
            | Self::WorldSetSign { .. }
            | Self::EntitySetDirection(..)
            | Self::EntitySetRotation(..)
            | Self::EntitySetPitch(..)
            | Self::EntityEventsClear(..)
            | Self::PlayerSetAbsPos(..)
            | Self::PlayerSetDirection(..)
            | Self::PlayerSetPitch(..)
            | Self::PlayerEventsClear
            | Self::EventsClear
            | Self::CustomLogDebug(..)
            | Self::CustomLogInfo(..)
            | Self::CustomLogWarn(..)
            | Self::CustomLogErr(..)
            | Self::CustomInventoryUnsafeGive { .. }
            | Self::CustomInventoryGive { .. }
            | Self::CustomOverrideReset
            | Self::CustomOverride { .. }
            | Self::CustomPostClient(..)
            | Self::CustomPostNoPrefix(..)
            | Self::CustomKeyPress(..)
            | Self::CustomKeyRelease(..)
            | Self::CustomWorldParticle { .. }
            | Self::CustomPlayerSetHealth(..)
            | Self::CustomPlayerCloseGUI
            | Self::CustomEntitySetAge { .. }
            | Self::CustomEntitySetSheepColor { .. }
            | Self::PlayerSetRotation(..)
            | Self::WorldSpawnParticle { .. }
            | Self::BlockSetLightLevel { .. }
            | Self::EntitySetDimension { .. }
            | Self::PlayerSetDimension { .. }
            | Self::CameraSetFollow { .. }
            | Self::CameraSetNormal { .. }
            | Self::CameraSetThirdPerson { .. }
            | Self::CameraSetDebug
            | Self::CameraSetDistance(..) => false,

            Self::EntityGetPos(_)
            | Self::EntityGetTile(_)
            | Self::PlayerGetPos
            | Self::PlayerGetTile
            | Self::WorldGetBlock(_)
            | Self::WorldGetBlocks(_, _)
            | Self::WorldGetBlockWithData(_)
            | Self::WorldGetHeight(_)
            | Self::WorldGetPlayerIds
            | Self::RaspberryJuiceWorldSpawnEntity { .. }
            | Self::RaspberryJamWorldSpawnEntity { .. }
            | Self::WorldGetEntities(..)
            | Self::WorldGetPlayerId(..)
            | Self::WorldRemoveEntities(..)
            | Self::WorldRemoveEntity(..)
            | Self::WorldGetEntityTypes
            | Self::EntityGetName(..)
            | Self::EntityGetDirection(..)
            | Self::EntityGetPitch(..)
            | Self::EntityGetRotation(..)
            | Self::EntityEventsBlockHits(..)
            | Self::EntityEventsChatPosts(..)
            | Self::EntityEventsProjectileHits(..)
            | Self::EntityGetEntities { .. }
            | Self::EntityRemoveEntities { .. }
            | Self::PlayerGetAbsPos
            | Self::PlayerGetDirection
            | Self::PlayerGetRotation
            | Self::PlayerGetPitch
            | Self::PlayerGetEntities { .. }
            | Self::PlayerRemoveEntities { .. }
            | Self::PlayerEventsBlockHits
            | Self::PlayerEventsChatPosts
            | Self::PlayerEventsProjectileHits
            | Self::EventsChatPosts
            | Self::EventsProjectileHits
            | Self::EventsBlockHits
            | Self::CustomInventoryGetSlot
            | Self::CustomUsernameAll
            | Self::CustomWorldDir
            | Self::CustomWorldName
            | Self::CustomWorldServername
            | Self::CustomPlayerGetHealth
            | Self::CustomPlayerGetGamemode
            | Self::CustomEntitySpawn { .. }
            | Self::EventsChatSize
            | Self::CustomRebornVersion
            | Self::CustomRebornFeature(..)
            | Self::EntityGetAllEntities
            | Self::WorldGetBlocksWithData { .. }
            | Self::BlockGetLightLevel { .. }
            | Self::EntityGetNameAndUUID(..)
            | Self::PlayerGetNameAndUUID
            | Self::CameraGetEntityId => true,
        }
    }
}

impl<'a> Display for Command<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn optional<T: Display>(param: &Option<T>, comma: bool) -> String {
            match param {
                Some(value) => format!("{}{value}", comma.then_some(",").unwrap_or_default()),
                None => String::new(),
            }
        }
        fn point<T: Display + Scalar, const D: usize>(param: &Point<T, D>) -> String {
            param
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        }

        match self {
            Self::CameraModeSetFixed => writeln!(f, "camera.mode.setFixed()"),
            Self::CameraModeSetFollow { target } => {
                writeln!(f, "camera.mode.setFollow({})", optional(target, false))
            }
            Self::CameraModeSetNormal { target } => {
                writeln!(f, "camera.mode.setNormal({})", optional(target, false))
            }
            Self::CameraSetPos(pos) => {
                writeln!(f, "camera.setPos({})", point(pos))
            }
            Self::CameraModeSetThirdPerson { target } => {
                writeln!(f, "camera.mode.setThirdPerson({})", optional(target, false))
            }
            Self::ChatPost(message) => {
                writeln!(f, "chat.post({message})")
            }
            Self::EntityGetPos(entity_id) => {
                writeln!(f, "entity.getPos({entity_id})")
            }
            Self::EntityGetTile(entity_id) => {
                writeln!(f, "entity.getTile({entity_id})")
            }
            Self::EntitySetPos(entity_id, pos) => {
                writeln!(f, "entity.setPos({entity_id},{})", point(pos))
            }
            Self::EntitySetTile(entity_id, tile) => {
                writeln!(f, "entity.setTile({entity_id},{})", point(tile))
            }
            Self::PlayerGetPos => {
                writeln!(f, "player.getPos()")
            }
            Self::PlayerGetTile => {
                writeln!(f, "player.getTile()")
            }
            Self::PlayerSetPos(pos) => {
                writeln!(f, "player.setPos({})", point(pos))
            }
            Self::PlayerSetTile(tile) => {
                writeln!(f, "player.setTile({})", point(tile))
            }
            Self::PlayerSetting { key, value } => {
                writeln!(f, "player.setting({key},{})", *value as i32)
            }
            Self::WorldCheckpointRestore => {
                writeln!(f, "world.checkpoint.restore()")
            }
            Self::WorldCheckpointSave => {
                writeln!(f, "world.checkpoint.save()")
            }
            Self::WorldGetBlock(pos) => {
                writeln!(f, "world.getBlock({})", point(pos))
            }
            Self::WorldGetBlocks(pos_1, pos_2) => {
                writeln!(f, "world.getBlocks({},{})", point(pos_1), point(pos_2))
            }
            Self::WorldGetPlayerId(name) => {
                writeln!(f, "world.getPlayerId({})", optional(name, false))
            }
            Self::WorldGetEntities(entity_type) => {
                writeln!(f, "world.getEntities({})", optional(entity_type, false))
            }
            Self::WorldRemoveEntity(entity_id) => {
                writeln!(f, "world.removeEntity({entity_id})")
            }
            Self::WorldRemoveEntities(entity_type) => {
                writeln!(f, "world.removeEntities({})", optional(entity_type, false))
            }
            Self::WorldGetBlockWithData(pos) => {
                writeln!(f, "world.getBlockWithData({})", point(pos))
            }
            Self::WorldGetHeight(pos) => {
                writeln!(f, "world.getHeight({})", point(pos))
            }
            Self::WorldGetPlayerIds => {
                writeln!(f, "world.getPlayerIds()")
            }
            Self::WorldSetBlock {
                coords,
                tile: block,
                data,
                json_nbt,
            } => {
                writeln!(
                    f,
                    "world.setBlock({},{block},{data}{})",
                    point(coords),
                    optional(json_nbt, true)
                )
            }
            Self::WorldSetBlocks {
                coords_1,
                coords_2,
                tile: block,
                data,
                json_nbt,
            } => {
                writeln!(
                    f,
                    "world.setBlocks({},{},{block},{data}{})",
                    point(coords_1),
                    point(coords_2),
                    optional(json_nbt, true)
                )
            }
            Self::WorldSetting { key, value } => {
                writeln!(f, "world.setting({key},{})", *value as i32)
            }
            Self::WorldSetSign {
                coords,
                tile: block,
                data,
                lines,
            } => {
                writeln!(
                    f,
                    "world.setSign({},{block},{data},{})",
                    point(coords),
                    lines
                        .iter()
                        .map(|line| line.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            Self::RaspberryJuiceWorldSpawnEntity {
                coords,
                entity_type,
            } => {
                writeln!(f, "world.spawnEntity({},{entity_type})", point(coords))
            }
            Self::WorldGetEntityTypes => {
                writeln!(f, "world.getEntityTypes()")
            }
            Self::EntityGetName(entity_id) => {
                writeln!(f, "entity.getName({entity_id})")
            }
            Self::EntityGetDirection(entity_id) => {
                writeln!(f, "entity.getDirection({entity_id})")
            }
            Self::EntitySetDirection(entity_id, direction) => {
                writeln!(f, "entity.setDirection({entity_id},{})", point(direction))
            }
            Self::EntityGetPitch(entity_id) => {
                writeln!(f, "entity.getPitch({entity_id})")
            }
            Self::EntitySetPitch(entity_id, pitch) => {
                writeln!(f, "entity.setPitch({entity_id},{pitch})")
            }
            Self::EntityGetRotation(entity_id) => {
                writeln!(f, "entity.getRotation({entity_id})")
            }
            Self::EntitySetRotation(entity_id, rotation) => {
                writeln!(f, "entity.setRotation({entity_id},{rotation})")
            }
            Self::EntityEventsClear(entity_id) => {
                writeln!(f, "entity.events.clear({entity_id})")
            }
            Self::EntityEventsBlockHits(entity_id) => {
                writeln!(f, "entity.events.block.hits({entity_id})")
            }
            Self::EntityEventsChatPosts(entity_id) => {
                writeln!(f, "entity.events.chat.posts({entity_id})")
            }
            Self::EntityEventsProjectileHits(entity_id) => {
                writeln!(f, "entity.events.projectile.hits({entity_id})")
            }
            Self::EntityGetEntities {
                target,
                distance,
                entity_type,
            } => {
                writeln!(
                    f,
                    "entity.getEntities({target},{distance}{})",
                    optional(entity_type, true)
                )
            }
            Self::EntityRemoveEntities {
                target,
                distance,
                entity_type,
            } => {
                writeln!(
                    f,
                    "entity.removeEntities({target},{distance}{})",
                    optional(entity_type, true)
                )
            }
            Self::PlayerGetAbsPos => {
                writeln!(f, "player.getAbsPos()")
            }
            Self::PlayerSetAbsPos(pos) => {
                writeln!(f, "player.setAbsPos({})", point(pos))
            }
            Self::PlayerGetDirection => {
                writeln!(f, "player.getDirection()")
            }
            Self::PlayerSetDirection(direction) => {
                writeln!(f, "player.setDirection({})", point(direction))
            }
            Self::PlayerGetRotation => {
                writeln!(f, "player.getRotation()")
            }
            Self::PlayerSetRotation(rotation) => {
                writeln!(f, "player.setRotation({rotation})")
            }
            Self::PlayerGetPitch => {
                writeln!(f, "player.getPitch()")
            }
            Self::PlayerSetPitch(pitch) => {
                writeln!(f, "player.setPitch({pitch})")
            }
            Self::PlayerEventsClear => {
                writeln!(f, "player.events.clear()")
            }
            Self::PlayerEventsBlockHits => {
                writeln!(f, "player.events.block.hits()")
            }
            Self::PlayerEventsChatPosts => {
                writeln!(f, "player.events.chat.posts()")
            }
            Self::PlayerEventsProjectileHits => {
                writeln!(f, "player.events.projectile.hits()")
            }
            Self::PlayerGetEntities {
                distance,
                entity_type,
            } => {
                writeln!(
                    f,
                    "player.getEntities({distance}{})",
                    optional(entity_type, true)
                )
            }
            Self::PlayerRemoveEntities {
                distance,
                entity_type,
            } => {
                writeln!(
                    f,
                    "player.removeEntities({distance}{})",
                    optional(entity_type, true)
                )
            }
            Self::EventsChatPosts => {
                writeln!(f, "events.chat.posts()")
            }
            Self::EventsProjectileHits => {
                writeln!(f, "events.projectile.hits()")
            }
            Self::EventsBlockHits => {
                writeln!(f, "events.block.hits()")
            }
            Self::EventsClear => {
                writeln!(f, "events.clear()")
            }
            Self::CustomLogDebug(message) => {
                writeln!(f, "custom.log.debug({message})")
            }
            Self::CustomLogInfo(message) => {
                writeln!(f, "custom.log.info({message})")
            }
            Self::CustomLogWarn(message) => {
                writeln!(f, "custom.log.warn({message})")
            }
            Self::CustomLogErr(message) => {
                writeln!(f, "custom.log.err({message})")
            }
            Self::CustomInventoryGetSlot => {
                writeln!(f, "custom.inventory.getSlot()")
            }
            Self::CustomInventoryUnsafeGive {
                id,
                auxillary,
                count,
            } => {
                writeln!(
                    f,
                    "custom.inventory.unsafeGive({},{},{})",
                    optional(id, false),
                    optional(auxillary, true),
                    optional(count, true)
                )
            }
            Self::CustomInventoryGive {
                id,
                auxillary,
                count,
            } => {
                writeln!(
                    f,
                    "custom.inventory.give({},{},{})",
                    optional(id, false),
                    optional(auxillary, true),
                    optional(count, true)
                )
            }
            Self::CustomOverrideReset => {
                writeln!(f, "custom.override.reset()")
            }
            Self::CustomOverride { before, after } => {
                writeln!(f, "custom.override({before},{after})")
            }
            Self::CustomPostClient(message) => {
                writeln!(f, "custom.post.client({message})")
            }
            Self::CustomPostNoPrefix(message) => {
                writeln!(f, "custom.post.noPrefix({message})")
            }
            Self::CustomKeyPress(key) => {
                writeln!(f, "custom.key.press({key})")
            }
            Self::CustomKeyRelease(key) => {
                writeln!(f, "custom.key.release({key})")
            }
            Self::CustomUsernameAll => {
                writeln!(f, "custom.username.all()")
            }
            Self::CustomWorldParticle { particle, coords } => {
                writeln!(f, "custom.world.particle({particle},{})", point(coords))
            }
            Self::CustomWorldDir => {
                writeln!(f, "custom.world.dir()")
            }
            Self::CustomWorldName => {
                writeln!(f, "custom.world.name()")
            }
            Self::CustomWorldServername => {
                writeln!(f, "custom.world.servername()")
            }
            Self::CustomPlayerGetHealth => {
                writeln!(f, "custom.player.getHealth()")
            }
            Self::CustomPlayerSetHealth(health) => {
                writeln!(f, "custom.player.setHealth({health})")
            }
            Self::CustomPlayerCloseGUI => {
                writeln!(f, "custom.player.closeGUI()")
            }
            Self::CustomPlayerGetGamemode => {
                writeln!(f, "custom.player.getGamemode()")
            }
            Self::CustomEntitySpawn {
                entity_type,
                health,
                coords,
                direction,
            } => {
                writeln!(
                    f,
                    "custom.entity.spawn({},{},{},{},{})",
                    entity_type.0,
                    point(coords),
                    health,
                    point(direction),
                    entity_type.1
                )
            }
            Self::CustomEntitySetAge { entity_id, age } => {
                writeln!(f, "custom.entity.setAge({entity_id},{age})")
            }
            Self::CustomEntitySetSheepColor { entity_id, color } => {
                writeln!(f, "custom.entity.setSheepColor({entity_id},{color})")
            }
            Self::EventsChatSize => {
                writeln!(f, "events.chat.size()")
            }
            Self::CustomRebornVersion => {
                writeln!(f, "custom.reborn.version()")
            }
            Self::CustomRebornFeature(feature) => {
                writeln!(f, "custom.reborn.feature({feature})")
            }
            Self::EntityGetAllEntities => {
                writeln!(f, "entity.getAllEntities()")
            }
            Self::WorldGetBlocksWithData { coords_1, coords_2 } => {
                writeln!(
                    f,
                    "world.getBlocksWithData({},{})",
                    point(coords_1),
                    point(coords_2)
                )
            }
            Self::BlockGetLightLevel { tile: block } => {
                writeln!(f, "block.getLightLevel({block})")
            }
            Self::BlockSetLightLevel { tile: block, level } => {
                writeln!(f, "block.setLightLevel({block},{level})")
            }
            Self::EntitySetDimension {
                entity_id,
                dimension,
            } => {
                writeln!(f, "entity.setDimension({entity_id},{dimension})")
            }
            Self::EntityGetNameAndUUID(entity_id) => {
                writeln!(f, "entity.getNameAndUUID({entity_id})")
            }
            Self::RaspberryJamWorldSpawnEntity {
                entity_type,
                coords,
                json_nbt,
            } => {
                writeln!(
                    f,
                    "world.spawnEntity({entity_type},{}{})",
                    point(coords),
                    optional(json_nbt, true)
                )
            }
            Self::PlayerSetDimension { dimension } => {
                writeln!(f, "player.setDimension({dimension})")
            }
            Self::PlayerGetNameAndUUID => {
                writeln!(f, "player.getNameAndUUID()")
            }
            Self::CameraGetEntityId => {
                writeln!(f, "camera.getEntityId()")
            }
            Self::CameraSetFollow { target } => {
                writeln!(f, "camera.setFollow({})", optional(target, false))
            }
            Self::CameraSetNormal { target } => {
                writeln!(f, "camera.setNormal({})", optional(target, false))
            }
            Self::CameraSetThirdPerson { target } => {
                writeln!(f, "camera.setThirdPerson({})", optional(target, false))
            }
            Self::CameraSetDebug => {
                writeln!(f, "camera.setDebug()")
            }
            Self::CameraSetDistance(distance) => {
                writeln!(f, "camera.setDistance({distance})")
            }
            Self::WorldSpawnParticle {
                particle,
                coords,
                direction,
                speed,
                count,
            } => {
                writeln!(
                    f,
                    "world.spawnParticle({particle},{},{},{},{})",
                    point(coords),
                    point(direction),
                    speed,
                    count
                )
            }
        }
    }
}

/// An error that occurs when an ApiString is created that contains a LF (line feed) character.
#[derive(Debug, Snafu)]
#[snafu(display("String must not contain LF characters."))]
pub struct NewlineStrError;

/// A string that does not contain the LF (line feed) character.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ApiStr<'a>(&'a str);

impl<'a> Deref for ApiStr<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Display for ApiStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<'a> ApiStr<'a> {
    pub const AUTOJUMP: Self = Self("autojump");

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

    /// Creates a new ApiString from the given string without checking for LF characters.
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

#[derive(Debug, Snafu)]
pub enum ChatStringError {
    #[snafu(display("{source}"), context(false))]
    Newline {
        source: NewlineStrError,
    },
    CP437,
}

/// A CP437 string that does not contain the LF (line feed) character.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ChatString(Vec<u8>);

impl AsRef<[u8]> for ChatString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Display for ChatString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = std::str::from_utf8(&self.0).unwrap();
        write!(f, "{string}")
    }
}

impl FromStr for ChatString {
    type Err = ChatStringError;

    /// Creates a new ApiString from the given string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains a LF (line feed) character.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('\n') {
            Err(NewlineStrSnafu.build().into())
        } else {
            let cp437 = str_to_cp437(s).context(CP437Snafu)?;
            Ok(Self(cp437))
        }
    }
}

impl ChatString {
    /// Creates a new ApiString from the given string.
    ///
    /// Invalid characters are replaced with the "?" character.
    #[must_use]
    pub fn from_str_lossy(inner: &str) -> Self {
        let cp437 = str_to_cp437_lossy(inner);
        Self(cp437)
    }

    /// Creates a new ApiString from the given bytes.
    ///
    /// # Safety
    ///
    /// The string must not be CP437-encoded and not contain LF (line feed) characters.
    #[must_use]
    pub unsafe fn from_vec_unchecked(inner: Vec<u8>) -> Self {
        Self(inner)
    }

    pub fn to_utf8(&self) -> String {
        cp437_to_string(&self.0)
    }
}

// MARK: Connection

/// An error that can occur when interacting with a Minecraft: Pi Edition game server.
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
    #[snafu(display("Failed to queue request: channel closed"))]
    Send { backtrace: Backtrace },
    #[snafu(display("Request failed: {source}"), context(false))]
    Recv {
        source: RecvError,
        backtrace: Backtrace,
    },
    #[snafu(display("Request queue full"))]
    QueueFull { backtrace: Backtrace },
}

/// Options that can be set to change the behavior of the connection to the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectOptions {
    /// The amount of time to wait for a response from the server before giving up.
    /// Setting this to a higher value may slow performance,
    /// but has a smaller chance of causing a timeout error.
    ///
    /// Defaults to 1 second.
    pub response_timeout: Option<Duration>,
    /// Whether to always wait for a response from the server.
    ///
    /// Because the server (under normal circumstances) does not acknowledge commands that do not require a
    /// response, the default behavior is to not check for a response for those commands. However,
    /// in an error scenario, the server may respond with a 'Fail' message, which would be missed if
    /// this setting is disabled.
    ///
    /// Enabling this setting will significantly degrade performance because commands that do not require a response
    /// will need to wait [`response_timeout`] seconds before continuing.
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
pub trait Protocol: Clone + Debug {
    /// Updates the connection options.
    fn set_options(&mut self, options: ConnectOptions) -> Result<(), ConnectionError>;
    /// Sends a command to the server and returns its response.
    ///
    /// If the command does not expect a response (as determined by [`Command::has_response`])
    /// and the [`ConnectOptions::always_wait_for_response`] option is not enabled,
    /// an empty string is returned without waiting for the server to respond.
    ///
    /// The operation will time out after the duration specified in the [`ConnectOptions::response_timeout`] option.
    ///
    /// Server responses are returned without processing or parsing.
    fn send(
        &self,
        command: Command<'_>,
    ) -> impl Future<Output = Result<String, ConnectionError>> + Send;

    /// Flushes the connection and disconnects.
    fn close(self) -> impl Future<Output = Result<(), ConnectionError>> + Send;

    /// Returns the percent of the transmission queue that is full.
    fn pressure(&self) -> f64;
}

/// A connection to a game server using the Minecraft: Pi Edition API protocol.
#[derive(Debug)]
pub struct ServerConnection {
    socket: BufWriter<TcpStream>,
    buffer: String,
    options: ConnectOptions,
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

    pub fn set_options(&mut self, options: ConnectOptions) -> Result<(), ConnectionError> {
        self.options = options;
        Ok(())
    }

    pub async fn send(&mut self, command: Command<'_>) -> Result<String, ConnectionError> {
        self.send_raw(command.to_string().as_bytes(), command.has_response())
            .await
    }

    pub async fn close(mut self) -> Result<(), ConnectionError> {
        self.socket.shutdown().await?;
        Ok(())
    }

    /// Sends a raw command to the server.
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
                assert!(has_response, "Using the `always_wait_for_response` setting without a `response_timeout` for a command that does not expect a response may cause an infinite hang.");
                self.read_frame().await
            }
        } else {
            Ok(String::new())
        }
    }

    /// Recieve a frame from the connection by either using data that has already been recieved
    /// or waiting for more data from the socket.
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

    /// Attempt to parse a frame from data that has already been recieved.
    pub(crate) fn parse_frame(&mut self) -> Option<String> {
        let idx = self.buffer.find('\n')?;
        let frame = self.buffer.drain(..idx + 1).collect();
        Some(frame)
    }
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_str_accepts_valid_strings() {
        let _ = super::ApiStr::new("hello").unwrap();
    }

    #[test]
    fn api_str_rejects_invalid_strings() {
        assert!(super::ApiStr::new("hello\n").is_err());
    }

    #[test]
    const fn api_str_unchecked_accepts_invalid_strings() {
        let _ = unsafe { super::ApiStr::new_unchecked("hello\n") };
    }

    #[test]
    fn api_str_and_chat_string_allow_special_characters() {
        let string = ChatString::from_str_lossy("I am so happy ♥");
        let command = Command::ChatPost(&string);
        assert_eq!(command.to_string(), "chat.post(I am so happy \u{003})\n");
        let string = ApiStr::new("I am so happy ♥").unwrap();
        assert_eq!(string.to_string(), "I am so happy ♥");
    }

    #[test]
    fn chat_post_formatting() {
        let string = ChatString::from_str_lossy("Hello world. This is a \"quote.\" )");
        let command = Command::ChatPost(&string);
        assert_eq!(
            command.to_string(),
            "chat.post(Hello world. This is a \"quote.\" ))\n"
        );
    }

    #[test]
    fn command_point_large_values() {
        let vec = Point3::new(1e100, 2.0, 3.0);
        let command = Command::PlayerSetPos(vec);
        assert_eq!(command.to_string(), "player.setPos(10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,2,3)\n");
    }

    #[test]
    fn command_point_serializes_f64_int() {
        let vec = Point3::new(1.0, 2.0, 3.0);
        let command = Command::PlayerSetPos(vec);
        assert_eq!(command.to_string(), "player.setPos(1,2,3)\n");
    }

    #[test]
    fn command_point_serializes_f64_real() {
        let vec = Point3::new(1.5, 2.5, 3.5);
        let command = Command::PlayerSetPos(vec);
        assert_eq!(command.to_string(), "player.setPos(1.5,2.5,3.5)\n");
    }

    #[test]
    fn command_point_serializes_i16() {
        let vec = Point3::new(1, 2, 3);
        let command = Command::WorldGetBlock(vec);
        assert_eq!(command.to_string(), "world.getBlock(1,2,3)\n");
    }

    #[test]
    fn command_point_serializes_i16_range() {
        let vec_1 = Point3::new(1, 2, 3);
        let vec_2 = Point3::new(4, 5, 6);
        let command = Command::WorldGetBlocks(vec_1, vec_2);
        assert_eq!(command.to_string(), "world.getBlocks(1,2,3,4,5,6)\n");
    }

    #[test]
    fn command_optionals_comma_ommitted_when_first_arg_none() {
        let command = Command::CameraModeSetFollow { target: None };
        assert_eq!(command.to_string(), "camera.mode.setFollow()\n");
    }

    #[test]
    fn command_optionals_comma_ommitted_when_first_arg_some() {
        let command = Command::CameraModeSetFollow {
            target: Some(EntityId(1)),
        };
        assert_eq!(command.to_string(), "camera.mode.setFollow(1)\n");
    }

    #[test]
    fn raspberry_jam_camera_apis_have_no_mode() {
        let command = Command::CameraModeSetNormal { target: None };
        assert_eq!(command.to_string(), "camera.mode.setNormal()\n");
        let command = Command::CameraSetNormal { target: None };
        assert_eq!(command.to_string(), "camera.setNormal()\n");
        let command = Command::CameraModeSetThirdPerson { target: None };
        assert_eq!(command.to_string(), "camera.mode.setThirdPerson()\n");
        let command = Command::CameraSetThirdPerson { target: None };
        assert_eq!(command.to_string(), "camera.setThirdPerson()\n");
        let command = Command::CameraModeSetFollow { target: None };
        assert_eq!(command.to_string(), "camera.mode.setFollow()\n");
        let command = Command::CameraSetFollow { target: None };
        assert_eq!(command.to_string(), "camera.setFollow()\n");
    }

    #[test]
    fn mcpi_extras_entity_new_sheep() {
        let entity = MCPIExtrasEntityType::new_sheep(SheepColor(1));
        assert_eq!(entity, MCPIExtrasEntityType(MCPIExtrasEntityType::SHEEP, 1));
    }

    #[test]
    fn mcpi_extras_entity_new_arrow() {
        let entity = MCPIExtrasEntityType::new_arrow(true);
        assert_eq!(entity, MCPIExtrasEntityType(MCPIExtrasEntityType::ARROW, 1));
    }

    #[test]
    fn mcpi_extras_entity_new_dropped_block() {
        let entity = MCPIExtrasEntityType::new_item(Tile(1));
        assert_eq!(entity, MCPIExtrasEntityType(MCPIExtrasEntityType::ITEM, 1));
    }

    #[test]
    fn mcpi_extras_entity_new_falling_block() {
        let entity = MCPIExtrasEntityType::new_falling_tile(Tile(1));
        assert_eq!(
            entity,
            MCPIExtrasEntityType(MCPIExtrasEntityType::FALLING_TILE, 1)
        );
    }

    #[test]
    fn mcpi_extras_entity_new_tnt_duration() {
        let entity = MCPIExtrasEntityType::new_tnt(Duration::from_secs(1));
        assert_eq!(entity, MCPIExtrasEntityType(MCPIExtrasEntityType::TNT, 20));
    }

    #[test]
    fn mcpi_extras_entity_new_tnt_ticks() {
        let entity = MCPIExtrasEntityType::new_tnt_const(1);
        assert_eq!(entity, MCPIExtrasEntityType(MCPIExtrasEntityType::TNT, 1));
    }
}
