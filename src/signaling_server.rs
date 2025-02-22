use crate::messages::{
    ClientAnswer, ClientMessage, ClientOffer, ServerAnswer, ServerMessage, ServerOffer, ID,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::mpsc::{self, Sender};
use tokio_tungstenite::tungstenite::Message;

struct ConnectionManager {
    channels: HashMap<String, Sender<ServerMessage>>,
}

impl ConnectionManager {
    fn new() -> ConnectionManager {
        ConnectionManager {
            channels: HashMap::new(),
        }
    }

    fn init() -> Sender<ConnectionManagerCommand> {
        let mut manager = ConnectionManager::new();
        let (sender, mut receiver) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    ConnectionManagerCommand::InsertChannel((id, channel)) => {
                        manager.channels.insert(id, channel);
                    }
                    ConnectionManagerCommand::Send((id, message)) => {
                        if let Some(destination) = manager.channels.get(&id) {
                            destination.send(message).await.unwrap();
                        }
                    }
                }
            }
        });

        return sender;
    }
}

enum ConnectionManagerCommand {
    InsertChannel((String, Sender<ServerMessage>)),
    Send((String, ServerMessage)),
}

pub async fn init<A>(addr: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: ToSocketAddrs,
{
    let connection_manager = ConnectionManager::init();
    let listener = TcpListener::bind(&addr).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let stream = tokio_tungstenite::accept_async(stream).await?;
        let id = uuid::Uuid::new_v4().to_string();
        let (mut write, mut read) = stream.split();

        let (tx, mut rx) = mpsc::channel(32);
        let insert_command = ConnectionManagerCommand::InsertChannel((id.clone(), tx));
        connection_manager.send(insert_command).await?;

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let message = serde_json::to_string::<ServerMessage>(&message).unwrap();
                let message = Message::from(message);
                write.send(message).await.unwrap();
            }
        });

        let connection_manager = connection_manager.clone();
        tokio::spawn(async move {
            while let Some(Ok(message)) = read.next().await {
                let message = serde_json::from_str::<ClientMessage>(&message.to_string()).unwrap();
                let (destination, message) = handle_client_message(message, id.clone()).await;
                let send_command = ConnectionManagerCommand::Send((destination, message));
                connection_manager.send(send_command).await.unwrap();
            }
        });
    }

    Ok(())
}

async fn handle_client_message(value: ClientMessage, socket_id: String) -> (String, ServerMessage) {
    match value {
        ClientMessage::Answer(answer) => handle_answer(answer, socket_id).await,
        ClientMessage::Offer(offer) => handle_offer(offer, socket_id).await,
        ClientMessage::GetMyID => handle_get_my_id(socket_id).await,
    }
}

async fn handle_answer(answer: ClientAnswer, socket_id: String) -> (String, ServerMessage) {
    (
        answer.to.clone(),
        ServerMessage::Answer(ServerAnswer {
            from: socket_id,
            to: answer.to,
            sdp: answer.sdp,
        }),
    )
}

async fn handle_offer(offer: ClientOffer, socket_id: String) -> (String, ServerMessage) {
    (
        offer.to.clone(),
        ServerMessage::Offer(ServerOffer {
            from: socket_id,
            to: offer.to,
            sdp: offer.sdp,
        }),
    )
}

async fn handle_get_my_id(socket_id: String) -> (String, ServerMessage) {
    (socket_id.clone(), ServerMessage::ID(ID { id: socket_id }))
}
