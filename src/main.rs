mod messages;
mod signaling_server;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let address =
        std::env::var("SIGNALING_SERVER_ADDRESS").expect("SIGNALING_SERVER_ADDRESS must be set");
    signaling_server::init(address).await;
}
