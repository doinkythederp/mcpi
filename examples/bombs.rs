use std::error::Error;
use std::pin::pin;
use std::time::Duration;

use futures_util::TryStreamExt;
use mcpi::connection::Tile;
use mcpi::World;
use nalgebra::Vector3;
use tokio::time::sleep;

const REPLACEMENT_TILE: Tile = Tile::WOOL;
const BLINKS: usize = 5;
const EXPLOSION_RADIUS: i16 = 5;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    let mut world = World::connect(addr).await?;

    world
        .post("mcpi bombs example: use the sword tool to right click on a block and blow it up")
        .await?;

    let mut block_stream = pin!(world.block_hits(Duration::from_millis(100)));
    while let Some(hit) = block_stream.try_next().await? {
        let mut world = world.clone();
        tokio::spawn(async move {
            let block = world.get_block(hit.coords).await.unwrap();

            // Blink the exploding block a few times.
            for _ in 0..BLINKS {
                world.set_tile(hit.coords, REPLACEMENT_TILE).await.unwrap();
                sleep(Duration::from_millis(100)).await;
                world.set_block(hit.coords, &block).await.unwrap();
                sleep(Duration::from_millis(100)).await;
            }

            // Remove all blocks in a sphere around it.
            for x in -EXPLOSION_RADIUS..=EXPLOSION_RADIUS {
                for y in -EXPLOSION_RADIUS..=EXPLOSION_RADIUS {
                    for z in -EXPLOSION_RADIUS..=EXPLOSION_RADIUS {
                        if x.pow(2) + y.pow(2) + z.pow(2) <= EXPLOSION_RADIUS.pow(2) {
                            let offset = Vector3::new(x, y, z);
                            world
                                .set_tile(hit.coords + offset, Tile::AIR)
                                .await
                                .unwrap();
                        }
                    }
                }
            }
        });
    }

    Ok(())
}
