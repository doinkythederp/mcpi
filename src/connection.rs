//! Implementation of the Minecraft: Pi Edition protocol.
//!
//! Sources include:
//! - [Picraft docs](https://picraft.readthedocs.io/en/release-1.0/protocol.html)
//! - [Wiki.vg](https://wiki.vg/Minecraft_Pi_Protocol)
//! - [martinohanlon/Minecraft-Pi-API](https://github.com/martinohanlon/Minecraft-Pi-API/blob/master/api.md)
//! - [MCPI Revival Wiki](https://mcpirevival.miraheze.org/wiki/MCPI_Revival)
//!

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::time::Duration;

use nalgebra::{SVector, Vector2, Vector3};
use snafu::{Backtrace, Snafu};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};

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
    pub const LADDER: Self = Self(65);
    pub const COBBLESTONE_STAIRS: Self = Self(67);
    pub const WALL_SIGN: Self = Self(68);
    pub const IRON_DOOR: Self = Self(71);
    pub const REDSTONE_ORE: Self = Self(73);
    pub const LIT_REDSTONE_ORE: Self = Self(73);
    pub const SNOW: Self = Self(78);
    pub const ICE: Self = Self(79);
    pub const SNOW_BLOCK: Self = Self(80);
    pub const CACTUS: Self = Self(81);
    pub const CLAY: Self = Self(82);
    pub const SUGARCANE: Self = Self(83);
    pub const FENCE: Self = Self(85);
    pub const NETHERRACK: Self = Self(87);
    pub const GLOWSTONE: Self = Self(89);
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
    pub const LEAVES_CARRIED: Self = Self(254);
    // TODO: what is this?
    pub const STONE_1: Self = Self(255);
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

/// Extra data that can be attached to a block, specific to that block type.
///
/// For many blocks, this data is used to represent the block's state, such as growth stage or orientation.
/// These common values are available as associated constants.
///
/// When working with blocks that don't store any extra state, the TileData will not be used by the server, but can be
/// set and later retrieved by the API user.
///
/// See also: [Minecraft: Pi Edition Complete Block List](https://mcpirevival.miraheze.org/wiki/Minecraft:_Pi_Edition_Complete_Block_List)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    // NETHER_REACTOR_CORE
    pub const NETHER_REACTOR_CORE_NORMAL: Self = Self(0);
    pub const NETHER_REACTOR_CORE_ACTIVE: Self = Self(1);
    pub const NETHER_REACTOR_CORE_BURNED: Self = Self(2);
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerSettingKey<'a>(pub ApiStr<'a>);

impl PlayerSettingKey<'_> {
    /// Controls whether the player will automatically jump when walking into a block.
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldSettingKey<'a>(pub ApiStr<'a>);

impl WorldSettingKey<'_> {
    /// Controls whether players can edit the world (such as by placing or destroying blocks).
    pub const WORLD_IMMUTABLE: Self = Self(ApiStr("world_immutable"));
    /// Controls whether player name tags will be shown.
    pub const NAME_TAGS: Self = Self(ApiStr("name_tags"));
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        target: Option<i32>,
    },
    CameraModeSetNormal {
        target: Option<i32>,
    },
    // TODO: Test whether this works on vanilla
    CameraModeSetThirdPerson {
        target: Option<i32>,
    },
    CameraSetPos(Vector3<f64>),
    // Chat APIs
    ChatPost(ApiStr<'a>),
    // Entity APIs
    EntityGetPos(i32),
    EntityGetTile(i32),
    EntitySetPos(i32, Vector3<f64>),
    EntitySetTile(i32, Vector3<i16>),
    // Player APIs
    PlayerGetPos,
    PlayerGetTile,
    PlayerSetPos(Vector3<f64>),
    PlayerSetTile(Vector3<i16>),
    PlayerSetting {
        key: PlayerSettingKey<'a>,
        value: bool,
    },
    // World APIs
    WorldCheckpointRestore,
    WorldCheckpointSave,
    WorldGetBlock(Vector3<i16>),
    /// Has Raspberry Jam mod extension to get block with NBT data. Requires a world setting to be set.
    /// TODO: look into this
    WorldGetBlockWithData(Vector3<i16>),
    WorldGetHeight(Vector2<i16>),
    WorldGetPlayerIds,
    WorldSetBlock {
        coords: Vector3<i16>,
        block: u8,
        data: Option<u8>,
        /// Raspberry Jam mod extension to set block with NBT data.
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
    WorldGetBlocks(Vector3<i16>, Vector3<i16>),
    /// When using the Raspberry Jam mod, this can be set to [`None`] to get the current player's ID.
    WorldGetPlayerId(Option<ApiStr<'a>>),
    WorldGetEntities(Option<JavaEntityType>),
    WorldRemoveEntity(i32),
    WorldRemoveEntities(Option<JavaEntityType>),
    WorldSetBlocks {
        coords_1: Vector3<i16>,
        coords_2: Vector3<i16>,
        block: u8,
        data: Option<u8>,
        /// Raspberry Jam mod extension to add NBT data to the blocks being set.
        ///
        /// Set to [`None`] when using other servers.
        json_nbt: Option<ApiStr<'a>>,
    },
    WorldSetSign {
        coords: Vector3<i16>,
        block: Tile,
        data: TileData,
        lines: Vec<ApiStr<'a>>,
    },
    RaspberryJuiceWorldSpawnEntity {
        coords: Vector3<f64>,
        entity_type: JavaEntityType,
    },
    WorldGetEntityTypes,

    // Entity APIs
    EntityGetName(i32),
    EntityGetDirection(i32),
    EntitySetDirection(i32, Vector3<f64>),
    EntityGetPitch(i32),
    EntitySetPitch(i32, f32),
    EntityGetRotation(i32),
    EntitySetRotation(i32, f32),
    EntityEventsClear(i32),
    EntityEventsBlockHits(i32),
    EntityEventsChatPosts(i32),
    EntityEventsProjectileHits(i32),
    EntityGetEntities {
        target: i32,
        distance: i32,
        entity_type: Option<JavaEntityType>,
    },
    EntityRemoveEntities {
        target: i32,
        distance: i32,
        entity_type: Option<JavaEntityType>,
    },

    // Player APIs
    PlayerGetAbsPos,
    PlayerSetAbsPos(Vector3<f64>),
    PlayerSetDirection(Vector3<f64>),
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
        coords: Vector3<f32>,
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
        coords: Vector3<f32>,
        direction: Vector2<f32>, // TODO: is this the most correct type?
    },
    CustomEntitySetAge {
        entity_id: i32,
        age: i32,
    },
    CustomEntitySetSheepColor {
        entity_id: i32,
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
        coords_1: Vector3<i16>,
        coords_2: Vector3<i16>,
    },
    WorldSpawnParticle {
        particle: RaspberryJamParticle<'a>,
        coords: Vector3<f64>,
        direction: Vector3<f64>, // TODO: Unclear how to use this
        speed: f64,
        count: i32,
    },

    // Block APIs
    BlockGetLightLevel {
        block: Tile,
    },
    BlockSetLightLevel {
        block: Tile,
        level: f32,
    },

    // Entity APIs
    EntitySetDimension {
        entity_id: i32,
        dimension: Dimension,
    },
    EntityGetNameAndUUID(i32),
    RaspberryJamWorldSpawnEntity {
        entity_type: JavaEntityType,
        coords: Vector3<f64>,
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
        target: Option<i32>,
    },
    CameraSetNormal {
        target: Option<i32>,
    },
    CameraSetThirdPerson {
        target: Option<i32>,
    },
    CameraSetDebug,
    CameraSetDistance(f32),
}

impl Command<'_> {
    #[must_use]
    pub fn has_response(&self) -> bool {
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
        fn vector<T: Display, const D: usize>(param: &SVector<T, D>) -> String {
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
                writeln!(f, "camera.setPos({})", vector(pos))
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
                writeln!(f, "entity.setPos({entity_id},{})", vector(pos))
            }
            Self::EntitySetTile(entity_id, tile) => {
                writeln!(f, "entity.setTile({entity_id},{})", vector(tile))
            }
            Self::PlayerGetPos => {
                writeln!(f, "player.getPos()")
            }
            Self::PlayerGetTile => {
                writeln!(f, "player.getTile()")
            }
            Self::PlayerSetPos(pos) => {
                writeln!(f, "player.setPos({})", vector(pos))
            }
            Self::PlayerSetTile(tile) => {
                writeln!(f, "player.setTile({})", vector(tile))
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
                writeln!(f, "world.getBlock({})", vector(pos))
            }
            Self::WorldGetBlocks(pos_1, pos_2) => {
                writeln!(f, "world.getBlocks({},{})", vector(pos_1), vector(pos_2))
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
                writeln!(f, "world.getBlockWithData({})", vector(pos))
            }
            Self::WorldGetHeight(pos) => {
                writeln!(f, "world.getHeight({})", vector(pos))
            }
            Self::WorldGetPlayerIds => {
                writeln!(f, "world.getPlayerIds()")
            }
            Self::WorldSetBlock {
                coords,
                block,
                data,
                json_nbt,
            } => {
                writeln!(
                    f,
                    "world.setBlock({},{block}{}{})",
                    vector(coords),
                    if json_nbt.is_some() {
                        format!(",{}", data.unwrap_or(0))
                    } else {
                        optional(data, true)
                    },
                    optional(json_nbt, true)
                )
            }
            Self::WorldSetBlocks {
                coords_1,
                coords_2,
                block,
                data,
                json_nbt,
            } => {
                writeln!(
                    f,
                    "world.setBlocks({},{},{block}{}{})",
                    vector(coords_1),
                    vector(coords_2),
                    if json_nbt.is_some() {
                        format!(",{}", data.unwrap_or(0))
                    } else {
                        optional(data, true)
                    },
                    optional(json_nbt, true)
                )
            }
            Self::WorldSetting { key, value } => {
                writeln!(f, "world.setting({key},{})", *value as i32)
            }
            Self::WorldSetSign {
                coords,
                block,
                data,
                lines,
            } => {
                writeln!(
                    f,
                    "world.setSign({},{block},{data},{})",
                    vector(coords),
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
                writeln!(f, "world.spawnEntity({},{entity_type})", vector(coords))
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
                writeln!(f, "entity.setDirection({entity_id},{})", vector(direction))
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
                writeln!(f, "player.setAbsPos({})", vector(pos))
            }
            Self::PlayerGetDirection => {
                writeln!(f, "player.getDirection()")
            }
            Self::PlayerSetDirection(direction) => {
                writeln!(f, "player.setDirection({})", vector(direction))
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
                writeln!(f, "custom.world.particle({particle},{})", vector(coords))
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
                    vector(coords),
                    health,
                    vector(direction),
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
                    vector(coords_1),
                    vector(coords_2)
                )
            }
            Self::BlockGetLightLevel { block } => {
                writeln!(f, "block.getLightLevel({block})")
            }
            Self::BlockSetLightLevel { block, level } => {
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
                    vector(coords),
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
                    vector(coords),
                    vector(direction),
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
pub struct ApiStrError;

/// A string that does not contain the LF (line feed) character.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn new(inner: &'a str) -> Result<Self, ApiStrError> {
        if inner.contains('\n') {
            ApiStrSnafu.fail()
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
    pub unsafe fn new_unchecked(inner: &'a str) -> Self {
        Self(inner)
    }
}

// MARK: Connection

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
    GenericFail {
        backtrace: Backtrace,
    },
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
    ConnectionClosed {
        backtrace: Backtrace,
    },
}

/// Options that can be set to change the behavior of the connection to the game.
pub struct ConnectOptions {
    /// The time (in millisecods) to wait for a response before giving up.
    /// Setting this to a higher value may slow performance,
    /// but has a smaller chance of causing a timeout error.
    ///
    /// Defaults to 1 second.
    pub response_timeout: Duration,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            response_timeout: Duration::from_secs(1),
        }
    }
}

/// A connection to game server using with the Minecraft: Pi Edition API protocol.
pub struct ServerConnection {
    socket: BufWriter<TcpStream>,
    buffer: String,
}

impl From<BufWriter<TcpStream>> for ServerConnection {
    fn from(value: BufWriter<TcpStream>) -> Self {
        Self {
            socket: value,
            buffer: String::new(),
        }
    }
}

impl ServerConnection {
    /// Connects to the Minecraft: Pi Edition server at the given address.
    pub async fn new(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        let socket = TcpStream::connect(addr).await?;
        Ok(Self {
            socket: BufWriter::new(socket),
            buffer: String::new(),
        })
    }

    /// Sends a command to the server and returns its response.
    pub async fn send(&mut self, data: Command<'_>) -> Result<String, ConnectionError> {
        self.socket.write_all(data.to_string().as_bytes()).await?;
        self.socket.flush().await?;

        if !data.has_response() {
            return Ok(String::new());
        }

        self.read_frame().await
    }

    /// Sends a command to the server without waiting for a response.
    ///
    /// This should usually be avoided because it may leave a response pending which could be
    /// read by a future command expecting a response.
    pub async fn write_command(&mut self, data: Command<'_>) -> Result<(), ConnectionError> {
        self.socket.write_all(data.to_string().as_bytes()).await?;
        self.socket.flush().await?;
        Ok(())
    }

    /// Recieve a frame from the connection by either using data that has already been recieved
    /// or waiting for more data from the socket.
    pub async fn read_frame(&mut self) -> Result<String, ConnectionError> {
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
    pub fn parse_frame(&mut self) -> Option<String> {
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
    fn api_str_unchecked_accepts_invalid_strings() {
        let _ = unsafe { super::ApiStr::new_unchecked("hello\n") };
    }

    #[test]
    fn api_str_and_chat_allow_control_characters() {
        let command = Command::ChatPost(
            ApiStr::new(
                "CRLF line endings, common on Windows, are prefixed with the \r character.",
            )
            .unwrap(),
        );
        assert_eq!(
            command.to_string(),
            "chat.post(CRLF line endings, common on Windows, are prefixed with the \r character.)\n"
        );
    }

    #[test]
    fn chat_post_formatting() {
        let command =
            Command::ChatPost(ApiStr::new("Hello world. This is a \"quote.\" )").unwrap());
        assert_eq!(
            command.to_string(),
            "chat.post(Hello world. This is a \"quote.\" ))\n"
        );
    }

    #[test]
    fn command_vector_large_values() {
        let vec = Vector3::new(1e100, 2.0, 3.0);
        let command = Command::PlayerSetPos(vec);
        assert_eq!(command.to_string(), "player.setPos(10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000,2,3)\n");
    }

    #[test]
    fn command_vector_serializes_f64_int() {
        let vec = Vector3::new(1.0, 2.0, 3.0);
        let command = Command::PlayerSetPos(vec);
        assert_eq!(command.to_string(), "player.setPos(1,2,3)\n");
    }

    #[test]
    fn command_vector_serializes_f64_real() {
        let vec = Vector3::new(1.5, 2.5, 3.5);
        let command = Command::PlayerSetPos(vec);
        assert_eq!(command.to_string(), "player.setPos(1.5,2.5,3.5)\n");
    }

    #[test]
    fn command_vector_serializes_i16() {
        let vec = Vector3::new(1, 2, 3);
        let command = Command::WorldGetBlock(vec);
        assert_eq!(command.to_string(), "world.getBlock(1,2,3)\n");
    }

    #[test]
    fn command_vector_serializes_i16_range() {
        let vec_1 = Vector3::new(1, 2, 3);
        let vec_2 = Vector3::new(4, 5, 6);
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
        let command = Command::CameraModeSetFollow { target: Some(1) };
        assert_eq!(command.to_string(), "camera.mode.setFollow(1)\n");
    }

    #[test]
    fn command_optionals_comma_included_when_mid_arg_some() {
        let command = Command::WorldSetBlock {
            coords: Vector3::default(),
            block: 1,
            data: Some(2),
            json_nbt: None,
        };
        assert_eq!(command.to_string(), "world.setBlock(0,0,0,1,2)\n");
    }

    #[test]
    fn command_optionals_comma_ommitted_when_mid_arg_none() {
        let command = Command::WorldSetBlock {
            coords: Vector3::default(),
            block: 1,
            data: None,
            json_nbt: None,
        };
        assert_eq!(command.to_string(), "world.setBlock(0,0,0,1)\n");
    }

    #[test]
    fn command_set_block_includes_data_when_json_nbt_some() {
        let command = Command::WorldSetBlock {
            coords: Vector3::new(1, 2, 3),
            block: 4,
            data: None,
            json_nbt: Some(ApiStr::new("{\"key\": \"value\"}").unwrap()),
        };
        assert_eq!(
            command.to_string(),
            "world.setBlock(1,2,3,4,0,{\"key\": \"value\"})\n"
        );
    }

    #[test]
    fn command_set_blocks_includes_data_when_json_nbt_some() {
        let command = Command::WorldSetBlocks {
            coords_1: Vector3::new(1, 2, 3),
            coords_2: Vector3::new(4, 5, 6),
            block: 7,
            data: None,
            json_nbt: Some(ApiStr::new("{\"key\": \"value\"}").unwrap()),
        };
        assert_eq!(
            command.to_string(),
            "world.setBlocks(1,2,3,4,5,6,7,0,{\"key\": \"value\"})\n"
        );
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
