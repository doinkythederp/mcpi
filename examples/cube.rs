use mcpi::connection::{Command, ConnectOptions, ServerConnection, Tile, TileData};
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

    // Set all the blocks at once
    connection
        .send(Command::WorldSetBlocks {
            coords_1: Vector3::new(0, 25, 0),
            coords_2: Vector3::new(25, 50, 25),
            tile: Tile::SANDSTONE,
            data: TileData::default(),
            json_nbt: None,
        })
        .await
        .unwrap();
}
