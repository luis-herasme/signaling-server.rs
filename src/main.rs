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
    let args = Args::parse();

    if let Some(address) = args.address {
        println!("[ARG] ADDRESS: {}", address);
        signaling_server::init(address).await?;
    } else {
        dotenv().ok();
        let address = std::env::var("ADDRESS").expect("The ADDRESS environment variable is not set");
        println!("[ENV] ADDRESS: {}", address);
        signaling_server::init(address).await?;
    }

    Ok(())
}
