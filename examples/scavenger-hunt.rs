use std::error::Error;
use std::time::{Duration, Instant};

use mcpi::connection::Tile;
use mcpi::entity::Entity;
use mcpi::World;
use nalgebra::{distance, Vector3};
use rand::Rng;
use tokio::time::sleep;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    const HIDDEN_TILE: Tile = Tile::DIAMOND_ORE;
    const FOUND_TILE: Tile = Tile::DIAMOND_BLOCK;

    let mut world = World::connect(addr).await?;
    let mut rng = rand::thread_rng();

    world.post("mcpi scavenger hunt example").await?;

    // Start by hiding a block the player must find.
    // The block is hidden in a random location within a 50x50x50 cube centered on the player.
    let starting_player_tile = world.me().get_tile().await?;
    let hidden_tile = starting_player_tile
        + Vector3::new(
            rng.gen_range(0..=50),
            rng.gen_range(0..=50),
            rng.gen_range(0..=50),
        );
    world.set_tile(hidden_tile, HIDDEN_TILE).await?;

    let start_time = Instant::now();

    // Now continually update the player with "hotter" or "colder" messages
    // depending on their distance from the hidden block.
    loop {
        let player_tile = world.me().get_position().await?;
        let distance = distance(&player_tile, &hidden_tile.cast());
        if distance < 2.0 {
            break;
        } else if distance < 5.0 {
            world.post("You're on fire!").await?;
        } else if distance < 10.0 {
            world.post("You're very warm!").await?;
        } else if distance < 20.0 {
            world.post("You're warm!").await?;
        } else if distance < 30.0 {
            world.post("You're cold!").await?;
        } else {
            world
                .post(&format!(
                    "You're freezing! The block is {} blocks away.",
                    distance.round(),
                ))
                .await?;
        }

        sleep(Duration::from_secs(1)).await;
    }

    let elapsed = start_time.elapsed();
    world.post("You found it!").await?;
    world
        .post(&format!(
            "You found the block in {} minutes and {} seconds!",
            elapsed.as_secs() / 60,
            elapsed.as_secs() % 60
        ))
        .await?;
    world.set_tile(hidden_tile, FOUND_TILE).await?;
    world.disconnect().await?;

    Ok(())
}
