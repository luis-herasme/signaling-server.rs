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
