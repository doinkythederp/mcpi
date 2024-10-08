//! Creates a 25x25x25 cube using individual block placement commands.
//!
//! All in all, 15,625 commands will be sent to the server. (25^3)

use mcpi::connection::{Command, ConnectOptions, ServerConnection, Tile, TileData};
use nalgebra::Point3;

#[tokio::main]
pub async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    let mut connection = ServerConnection::new(addr, ConnectOptions::default())
        .await
        .unwrap();
    let coords_1 = Point3::new(0, 25, 0);
    let coords_2 = coords_1.map(|x| x + 25);

    for x in coords_1.x..coords_2.x {
        for y in coords_1.y..coords_2.y {
            for z in coords_1.z..coords_2.z {
                println!("Setting block at {:?}", Point3::new(x, y, z));
                connection
                    .send(Command::WorldSetBlock {
                        tile: Tile::SANDSTONE,
                        coords: Point3::new(x, y, z),
                        data: TileData::default(),
                        json_nbt: None,
                    })
                    .await
                    .unwrap();
            }
        }
    }
}
