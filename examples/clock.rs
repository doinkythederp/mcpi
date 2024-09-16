use std::error::Error;
use std::f64::consts::PI;
use std::time::Duration;

use chrono::{Local, Timelike};
use line_drawing::{Bresenham, BresenhamCircle};
use mcpi::connection::{Tile, TileData};
use mcpi::{Block, World};
use nalgebra::{Point3, Vector3};
use tokio::time::interval;

const CLOCK_TILE: Tile = Tile::GOLD_BLOCK;
const SECOND_HAND_BLOCK: Block = Block::new(Tile::WOOL, TileData::RED);
const MINUTE_HAND_BLOCK: Block = Block::new(Tile::WOOL, TileData::YELLOW);
const HOUR_HAND_BLOCK: Block = Block::new(Tile::WOOL, TileData::BLACK);

async fn draw_frame(world: &mut World, center: Point3<i16>, radius: i16) -> mcpi::Result {
    for (x, y) in BresenhamCircle::new(center.x, center.y, radius) {
        world
            .set_tile(Point3::new(x, y, center.z), CLOCK_TILE)
            .await?;
    }
    Ok(())
}

async fn draw_hand(
    world: &mut World,
    origin: Point3<i16>,
    len: f64,
    angle_rad: f64,
    block: &Block,
) -> mcpi::Result {
    let end_coords = origin
        + Vector3::new(
            (angle_rad.cos() * len).round() as i16,
            (angle_rad.sin() * len).round() as i16,
            0,
        );
    for (x, y) in Bresenham::new((origin.x, origin.y), (end_coords.x, end_coords.y)) {
        world.set_block(Point3::new(x, y, origin.z), block).await?;
    }
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    let mut world = World::connect(addr).await?;

    let center = Point3::new(0, 30, 0);
    let radius = 20;

    world.post("mcpi clock example").await?;

    let mut interval = interval(Duration::from_secs(1));

    loop {
        let elapsed = Local::now().time().num_seconds_from_midnight() as f64;
        let seconds = elapsed % 60.0;
        let minutes = (elapsed / 60.0) % 60.0;
        let hours = (elapsed / 3600.0) % 12.0;

        draw_frame(&mut world, center, radius).await?;
        draw_hand(
            &mut world,
            center,
            radius.into(),
            seconds * PI / 30.0,
            &SECOND_HAND_BLOCK,
        )
        .await?;
        draw_hand(
            &mut world,
            center,
            (radius - 2).into(),
            minutes * PI / 30.0,
            &MINUTE_HAND_BLOCK,
        )
        .await?;
        draw_hand(
            &mut world,
            center,
            (radius / 2).into(),
            hours * PI / 6.0,
            &HOUR_HAND_BLOCK,
        )
        .await?;

        interval.tick().await;
    }
}
