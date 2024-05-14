use std::pin::pin;
use std::time::Duration;

use futures_util::TryStreamExt;
use mcpi::World;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    let world = World::connect(addr).await?;

    let mut block_stream = pin!(world.block_hits(Duration::from_millis(5)));
    while let Some(hit) = block_stream.try_next().await? {
        let block = world.get_block(hit.coords).await?;
        println!(
            "{} block hit at {} (face: {:?}, player #{})",
            block.tile.display(),
            hit.coords,
            hit.face,
            hit.player_id
        );
    }

    Ok(())
}
