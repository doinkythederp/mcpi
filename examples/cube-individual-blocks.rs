//! Creates a 25x25x25 cube using individual block placement commands.
//!
//! All in all, 15,625 commands will be sent to the server. (50^3)

use mcpi::connection::{Command, ConnectOptions, Protocol, ServerConnection, Tile};
use nalgebra::Vector3;

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
    let coords_1 = Vector3::new(0, 25, 0);
    let coords_2 = coords_1.map(|x| x + 25);

    for x in coords_1.x..coords_2.x {
        for y in coords_1.y..coords_2.y {
            for z in coords_1.z..coords_2.z {
                println!("Setting block at {:?}", Vector3::new(x, y, z));
                connection
                    .send(Command::WorldSetBlock {
                        block: Tile::SANDSTONE.0,
                        coords: Vector3::new(x, y, z),
                        data: None,
                        json_nbt: None,
                    })
                    .await
                    .unwrap();
            }
        }
    }
}
