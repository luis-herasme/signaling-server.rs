use std::collections::HashMap;
use tokio::sync::mpsc::{self, Receiver, Sender};

struct Channels<T> {
    channels: HashMap<String, Sender<T>>,
}

impl<T: Send + Sync + 'static> Channels<T> {
    fn init() -> Sender<Command<T>> {
        let mut channels = Channels { channels: HashMap::new() };
        let (sender, mut receiver) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    Command::Insert((id, channel)) => {
                        channels.channels.insert(id, channel);
                    }
                    Command::Send((id, message)) => {
                        if let Some(destination) = channels.channels.get(&id) {
                            destination
                                .send(message)
                                .await
                                .expect("Peer could not send message to another peer");
                        }
                    }
                    Command::Remove(id) => {
                        channels.channels.remove(&id);
                    }
                }
            }
        });

        sender
    }
}

enum Command<T> {
    Insert((String, Sender<T>)),
    Send((String, T)),
    Remove(String),
}

pub struct ConnectionsHandler<T> {
    command_emitter: Sender<Command<T>>,
}

impl<T: Send + Sync + 'static> ConnectionsHandler<T> {
    pub fn new() -> ConnectionsHandler<T> {
        let command_emitter = Channels::init();
        ConnectionsHandler { command_emitter }
    }

    pub async fn create_connection(&self) -> (String, Receiver<T>) {
        let (sender, receiver) = mpsc::channel(32);
        let id = uuid::Uuid::new_v4().to_string();

        let insert_command = Command::Insert((id.clone(), sender));
        self.command_emitter
            .send(insert_command)
            .await
            .expect("Could not send (channel) connection-insert command");

        (id, receiver)
    }

    pub async fn send_message(&self, id: String, message: T) {
        let send_command = Command::Send((id, message));
        self.command_emitter
            .send(send_command)
            .await
            .expect("Could not send (channel) peer-message command");
    }

    pub async fn remove_connection(&self, id: String) {
        let remove_command = Command::Remove(id);
        self.command_emitter
            .send(remove_command)
            .await
            .expect("Could not send remove command");
    }

    pub fn clone(&self) -> ConnectionsHandler<T> {
        ConnectionsHandler {
            command_emitter: self.command_emitter.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionsHandler;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn can_handle_a_connection() {
        let connections: ConnectionsHandler<String> = ConnectionsHandler::new();

        // Create a connections
        let (id, mut receiver) = connections.create_connection().await;

        // Send message
        connections.send_message(id.clone(), String::from("hello")).await;

        // Verify message received
        assert_eq!(receiver.recv().await.unwrap(), String::from("hello"));
    }

    #[tokio::test]
    async fn can_handle_multiple_connections() {
        let connections: ConnectionsHandler<String> = ConnectionsHandler::new();

        // Create two connections
        let (id1, mut receiver1) = connections.create_connection().await;
        let (id2, mut receiver2) = connections.create_connection().await;

        // Send messages to both connections
        connections.send_message(id1.clone(), String::from("hello1")).await;
        connections.send_message(id2.clone(), String::from("hello2")).await;

        // Verify messages are received by correct receivers
        assert_eq!(receiver1.recv().await.unwrap(), String::from("hello1"));
        assert_eq!(receiver2.recv().await.unwrap(), String::from("hello2"));
    }

    #[tokio::test]
    async fn messages_preserve_order() {
        let connections: ConnectionsHandler<i32> = ConnectionsHandler::new();
        let (id, mut receiver) = connections.create_connection().await;

        // Send multiple messages
        for i in 0..5 {
            connections.send_message(id.clone(), i).await;
        }

        // Verify messages are received in order
        for i in 0..5 {
            assert_eq!(receiver.recv().await.expect("Could no receive channel message"), i);
        }
    }

    #[tokio::test]
    async fn nonexistent_id_message_is_ignored() {
        let connections: ConnectionsHandler<String> = ConnectionsHandler::new();

        // Send message to nonexistent connection
        connections.send_message(String::from("nonexistent"), String::from("hello")).await;

        // If we got here without panicking, the test passes
        // Add small delay to ensure message processing completed
        sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn can_handle_multiple_messages_multiple_connections() {
        let connections: ConnectionsHandler<String> = ConnectionsHandler::new();

        // Create three connections
        let (id1, mut receiver1) = connections.create_connection().await;
        let (id2, mut receiver2) = connections.create_connection().await;
        let (id3, mut receiver3) = connections.create_connection().await;

        // Send multiple messages to each connection
        for i in 0..3 {
            connections.send_message(id1.clone(), format!("conn1-msg-{}", i)).await;
            connections.send_message(id2.clone(), format!("conn2-msg-{}", i)).await;
            connections.send_message(id3.clone(), format!("conn3-msg-{}", i)).await;
        }

        // Verify all messages are received in order for each connection
        for i in 0..3 {
            assert_eq!(receiver1.recv().await.unwrap(), format!("conn1-msg-{}", i));
            assert_eq!(receiver2.recv().await.unwrap(), format!("conn2-msg-{}", i));
            assert_eq!(receiver3.recv().await.unwrap(), format!("conn3-msg-{}", i));
        }
    }
}
