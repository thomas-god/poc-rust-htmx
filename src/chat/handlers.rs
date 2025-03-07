use std::time::Instant;

use axum::{
    extract::{
        FromRequest, Request, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{StatusCode, header},
    response::IntoResponse,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use maud::{DOCTYPE, html};
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{
    ApiState,
    chat::{state::ChatMessage, templates::username_form},
};

use super::{
    state::ChatHistoryRequest,
    templates::{chat, new_chat_message},
};

pub async fn handle_chat_ws(
    State(state): State<ApiState>,
    WebsocketContentNegotiator(ws): WebsocketContentNegotiator,
) -> impl IntoResponse {
    if let Some(ws) = ws {
        let state = state.read().await;
        let tx_broadcast = state.chat.tx_broadcast.clone();
        let tx_history = state.chat.tx_history.clone();
        return ws
            .on_upgrade(move |socket| handle_socket(socket, tx_broadcast, tx_history))
            .into_response();
    }

    html!(
        (DOCTYPE)
        html {
            head {
                script src="/assets/htmx.min.js" {}
                script src="/assets/ws.min.js" {}
                link href="/assets/style/output.css" rel="stylesheet";
            }
            body {
                h1.text-2xl.font-bold.text-center.mt-2 { "Welcome to the chat" }
                div hx-ext="ws" ws-connect="/chat" {
                    (username_form())
                }
            }
        }
    )
    .into_response()
}

async fn handle_socket(
    socket: WebSocket,
    tx_broadcast: broadcast::Sender<ChatMessage>,
    tx_history: mpsc::Sender<ChatHistoryRequest>,
) {
    tokio::spawn(async move {
        handle_chat_connection(socket, tx_broadcast, tx_history).await;
    });
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum WSIncomingMessage {
    Username { username: String },
    NewMessage { content: String },
}

async fn handle_chat_connection(
    mut socket: WebSocket,
    tx_broadcast: broadcast::Sender<ChatMessage>,
    tx_history: mpsc::Sender<ChatHistoryRequest>,
) {
    // First, wait for user to supply a username
    let username = loop {
        if let Some(Ok(Message::Text(msg))) = socket.next().await {
            if let Ok(WSIncomingMessage::Username { username }) = serde_json::from_str(&msg) {
                break username;
            }
        };
    };

    if username.is_empty() {
        tracing::warn!("Username is empty, terminating processing this socket");
        return;
    }
    tracing::info!("Starting chat connection for {username:?}");

    let (sink, stream) = socket.split();
    let cloned_username = username.clone();
    let rx_broadcast = tx_broadcast.subscribe();
    let sink_handle =
        tokio::spawn(
            async move { process_sink(sink, cloned_username, rx_broadcast, tx_history).await },
        );
    let cloned_username = username.clone();
    let stream_handle =
        tokio::spawn(async move { process_stream(stream, cloned_username, tx_broadcast).await });

    tokio::select! {
        _ = sink_handle => {
            tracing::info!("Sink handle finished for {username:?}");
        }
        _ = stream_handle => {
            tracing::info!("Stream handle finished for {username:?}");
        }
    }
    tracing::info!("Chat connection for {username:?} finished");
}

pub struct WebsocketContentNegotiator(Option<WebSocketUpgrade>);

impl<S> FromRequest<S> for WebsocketContentNegotiator
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let upgrade = req
            .headers()
            .get(header::UPGRADE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if upgrade.starts_with("websocket") {
            let Ok(ws) = WebSocketUpgrade::from_request(req, state).await else {
                return Err(StatusCode::BAD_REQUEST);
            };
            return Ok(WebsocketContentNegotiator(Some(ws)));
        }

        Ok(WebsocketContentNegotiator(None))
    }
}

async fn process_sink(
    mut sink: SplitSink<WebSocket, Message>,
    username: String,
    mut rx_broadcast: broadcast::Receiver<ChatMessage>,
    tx_history: mpsc::Sender<ChatHistoryRequest>,
) {
    // Send a snapshot of the existing messages
    let (tx_back, rx) = oneshot::channel();
    let _ = tx_history.send(ChatHistoryRequest { tx_back }).await;

    match rx.await {
        Ok(messages) => {
            let _ = sink
                .send(Message::text(chat(&username, messages).into_string()))
                .await;
        }
        Err(err) => {
            tracing::error!(
                "Unable to send intitial messages, stoping processing websocket. {err:?}"
            );
            return;
        }
    };

    // Listen for new messages
    while let Ok(msg) = rx_broadcast.recv().await {
        let _ = sink
            .send(Message::text(
                new_chat_message(&username, msg).into_string(),
            ))
            .await;
    }
}

async fn process_stream(
    mut stream: SplitStream<WebSocket>,
    username: String,
    tx: broadcast::Sender<ChatMessage>,
) {
    while let Some(Ok(Message::Text(msg))) = stream.next().await {
        match serde_json::from_str::<WSIncomingMessage>(&msg) {
            Ok(WSIncomingMessage::Username { username }) => {
                tracing::info!("Username: {username:?}");
            }
            Ok(WSIncomingMessage::NewMessage { content }) => {
                let chat_message = ChatMessage {
                    content,
                    username: username.clone(),
                    timestamp: Instant::now(),
                };
                let _ = tx.send(chat_message);
            }
            Err(err) => tracing::error!("{err:?}"),
        }
    }
}
