use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use chat::{handlers::handle_chat_ws, state::ChatState};
use maud::{DOCTYPE, Markup, html};

use todos::{
    handlers::{create_todo, delete_todo, get_todos, toggle_todo},
    state::TodosState,
};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub mod chat;
pub mod todos;
pub mod utils;

pub struct AppState {
    todos: TodosState,
    chat: ChatState,
}
pub type ApiState = Arc<RwLock<AppState>>;

pub fn build_app() -> Router {
    let state = Arc::new(RwLock::new(AppState {
        todos: TodosState::new(),
        chat: ChatState::new(),
    }));

    Router::new()
        .route("/", get(root))
        .route("/todos", get(get_todos))
        .route("/todo", post(create_todo))
        .route("/todo/{id}/toggle", post(toggle_todo))
        .route("/todo/{id}", delete(delete_todo))
        .route("/chat", get(handle_chat_ws))
        .layer(TraceLayer::new_for_http())
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}

async fn root() -> Markup {
    html!(
        (DOCTYPE)
        html {
            head {
                script src="/assets/htmx.min.js" {}
                script src="/assets/ws.min.js" {}
                link href="/assets/style/output.css" rel="stylesheet";
            }
            body {
                h1.text-2xl.font-bold.text-center.mt-2 { "htmx + rust" }
                div.join.join-horizontal.mt-4.w-full.justify-center {
                    a.btn.btn-primary.join-item href="/todos" { "Todos" }
                    a.btn.btn-primary.join-item href="/chat" { "Chat" }
                }
            }
        }
    )
}
