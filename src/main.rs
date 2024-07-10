mod messages;
mod signaling_server;

#[tokio::main]
async fn main() {
    signaling_server::init("0.0.0.0:1234").await;
}
