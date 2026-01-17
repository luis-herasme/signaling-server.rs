use crate::connection_manager::ConnectionsHandler;
use crate::messages::{ClientMessage, ServerAnswer, ServerMessage, ServerOffer, ID};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_tungstenite::tungstenite::Message;

pub async fn init<A>(addr: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: ToSocketAddrs,
{
    let connection_handler = ConnectionsHandler::new();
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Signaling server started");

    while let Ok((stream, addr)) = listener.accept().await {
        let stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = stream.split();

        let (socket_id, mut receiver) = connection_handler.create_connection().await;
        log::info!("[{}] Connection established from {}", socket_id, addr);

        // Send messages to socket
        let connection_handler = connection_handler.clone();
        let socket_id_send = socket_id.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let Ok(message) = serde_json::to_string::<ServerMessage>(&message) else {
                    log::warn!("[{}] Failed to serialize message", socket_id_send);
                    continue;
                };

                let message = Message::from(message);

                if write.send(message).await.is_err() {
                    log::debug!("[{}] WebSocket send failed, closing sender", socket_id_send);
                    break;
                }
            }
        });

        // Handle received messages from socket
        let connection_handler = connection_handler.clone();
        tokio::spawn(async move {
            while let Some(Ok(message)) = read.next().await {
                let message_str = message.to_string();
                match serde_json::from_str::<ClientMessage>(&message_str) {
                    Ok(client_message) => {
                        let (destination_id, response) = handle_client_message(client_message, socket_id.clone());
                        connection_handler.send_message(destination_id, response).await;
                    }
                    Err(e) => {
                        log::warn!("[{}] Failed to deserialize message: {}", socket_id, e);
                    }
                }
            }

            log::info!("[{}] Connection closed", socket_id);
            connection_handler.remove_connection(socket_id).await;
        });
    }

    Ok(())
}

fn handle_client_message(message: ClientMessage, socket_id: String) -> (String, ServerMessage) {
    match message {
        ClientMessage::Offer(offer) => {
            log::debug!("[{}] Routing offer to {}", socket_id, offer.to);
            let message = ServerMessage::Offer(ServerOffer {
                from: socket_id,
                sdp: offer.sdp,
            });

            return (offer.to, message);
        }
        ClientMessage::Answer(answer) => {
            log::debug!("[{}] Routing answer to {}", socket_id, answer.to);
            let message = ServerMessage::Answer(ServerAnswer {
                from: socket_id,
                sdp: answer.sdp,
            });

            return (answer.to, message);
        }
        ClientMessage::GetMyID => {
            log::debug!("[{}] Providing ID to client", socket_id);
            let message = ServerMessage::ID(ID { id: socket_id.clone() });

            return (socket_id, message);
        }
    }
}
