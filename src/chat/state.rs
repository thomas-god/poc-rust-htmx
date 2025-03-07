use std::time::Instant;

use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct ChatState {
    pub tx_broadcast: broadcast::Sender<ChatMessage>,
    pub tx_history: mpsc::Sender<ChatHistoryRequest>,
}

impl ChatState {
    pub fn new() -> ChatState {
        let (tx_broadcast, rx_broadcast) = broadcast::channel(128);
        let tx_history = ChatHistory::start(rx_broadcast);
        ChatState {
            tx_broadcast,
            tx_history,
        }
    }
}

impl Default for ChatState {
    fn default() -> Self {
        ChatState::new()
    }
}

pub struct ChatHistoryRequest {
    pub tx_back: oneshot::Sender<Vec<ChatMessage>>,
}

pub struct ChatHistory {
    messages: Vec<ChatMessage>,
    rx_broadcast: broadcast::Receiver<ChatMessage>,
    rx_client: mpsc::Receiver<ChatHistoryRequest>,
}

impl ChatHistory {
    pub fn start(
        rx_broadcast: broadcast::Receiver<ChatMessage>,
    ) -> mpsc::Sender<ChatHistoryRequest> {
        let (tx, rx_client) = mpsc::channel(32);
        let mut chat_history = ChatHistory {
            messages: vec![ChatMessage {
                username: "Test user".to_owned(),
                content: "test message".to_owned(),
                timestamp: Instant::now(),
            }],
            rx_broadcast,
            rx_client,
        };

        tokio::spawn(async move {
            chat_history.run().await;
        });

        tx
    }

    pub async fn run(&mut self) {
        tracing::info!("Starting ChatHistory actor");

        loop {
            tokio::select! {
                Ok(msg) = self.rx_broadcast.recv() => {
                    self.messages.push(msg);
                }
                Some(ChatHistoryRequest { tx_back }) = self.rx_client.recv() => {
                    let idx = self.messages.len().saturating_sub(10);
                    let _ = tx_back.send(
                        self.messages[idx..].to_vec());
                }
            }
        }
    }
}
