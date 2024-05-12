use mcpi::World;

#[tokio::main]
pub async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };
    let message = match args.get(2) {
        Some(message) => message.as_ref(),
        None => "Hello, world!",
    };

    let mut connection = World::connect(addr).await.unwrap();

    connection.post(message).await.unwrap();
}
