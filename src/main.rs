mod connection_manager;
mod messages;
mod signaling_server;
use clap::Parser;
use dotenv::dotenv;

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        help = "The address to bind the signaling server to. Falls back to ADDRESS environment variable."
    )]
    address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let address = if let Some(address) = args.address {
        log::info!("Using address from argument: {}", address);
        address
    } else {
        dotenv().ok();
        let address = std::env::var("ADDRESS").expect("The ADDRESS environment variable is not set");
        log::info!("Using address from environment: {}", address);
        address
    };

    signaling_server::init(address).await?;

    Ok(())
}
