mod connection_manager;
mod messages;
mod signaling_server;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let address = std::env::var("ADDRESS").expect("The ADDRESS environment variable is not set");
    signaling_server::init(address).await?;

    Ok(())
}
