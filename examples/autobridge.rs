use std::error::Error;
use std::time::Duration;

use mcpi::connection::Tile;
use mcpi::entity::Entity;
use mcpi::{pos_to_tile, World};
use nalgebra::{distance, Vector3};
use tokio::time::sleep;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    const BRIDGE_BLOCK: Tile = Tile::DIAMOND_BLOCK;

    let mut world = World::connect(addr).await?;

    world.post("mcpi autobridge example").await?;

    let player = world.me();
    let mut last_player_pos = player.get_position().await?;
    loop {
        sleep(Duration::from_millis(10)).await;

        let player_pos = player.get_position().await?;

        if distance(&player_pos, &last_player_pos) > 0.2 {
            // The player is walking. Let's put a block under where they are about to step.

            // First figure out where the player is walking to by continuing their trajectory until they hit a new tile.
            let player_velocity = player_pos - last_player_pos;

            let player_tile = pos_to_tile(&player_pos);
            let mut predicted_next_tile = player_pos;
            while player_tile == pos_to_tile(&predicted_next_tile) {
                predicted_next_tile += player_velocity;
            }

            // Now place a block under the predicted next tile.
            let block_pos = pos_to_tile(&predicted_next_tile) - Vector3::new(0, 1, 0);
            world.set_tile(block_pos, BRIDGE_BLOCK).await?;
        }

        last_player_pos = player_pos;
    }
}
