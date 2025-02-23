use crate::connection_manager::ConnectionsHandler;
use crate::messages::{ClientAnswer, ClientMessage, ClientOffer, ServerAnswer, ServerMessage, ServerOffer, ID};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_tungstenite::tungstenite::Message;

pub async fn init<A>(addr: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: ToSocketAddrs,
{
    let connection_handler = ConnectionsHandler::new();
    let listener = TcpListener::bind(&addr).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = stream.split();

        let (socket_id, mut receiver) = connection_handler.create_connection().await;

        // Send messages to socket
        let connection_handler = connection_handler.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let message = serde_json::to_string::<ServerMessage>(&message).unwrap();
                let message = Message::from(message);
                write.send(message).await.unwrap();
            }
        });

        // Handle received messages from socket
        let connection_handler = connection_handler.clone();
        tokio::spawn(async move {
            while let Some(Ok(message)) = read.next().await {
                let message = serde_json::from_str::<ClientMessage>(&message.to_string()).unwrap();
                let (destination_id, message) = handle_client_message(message, socket_id.clone()).await;
                connection_handler.send_message(destination_id, message).await;
            }

            connection_handler.remove_connection(socket_id).await;
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
