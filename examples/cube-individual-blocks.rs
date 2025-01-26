//! Creates a 25x25x25 cube using individual block placement commands.
//!
//! All in all, 15,625 commands will be sent to the server. (25^3)

use mcpi::connection::Tile;
use mcpi::{Block, World};
use nalgebra::{Point3, Vector3};

const BLOCK: Block = Block::from_tile(Tile::SANDSTONE);
const CUBE_ORIGIN: Point3<i16> = Point3::new(0, 25, 0);
const CUBE_SIZE: Vector3<i16> = Vector3::new(25, 25, 25);

#[tokio::main]
pub async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    let coords_1 = CUBE_ORIGIN;
    let coords_2 = CUBE_ORIGIN + CUBE_SIZE;

    let mut world = World::connect(addr).await.unwrap();

    for x in coords_1.x..coords_2.x {
        for y in coords_1.y..coords_2.y {
            for z in coords_1.z..coords_2.z {
                let coords = Point3::new(x, y, z);
                println!("Setting block at {coords:?}");
                world.set_block(coords, &BLOCK).await.unwrap();
            }
        }
    }
}
