use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use chat::{handlers::handle_chat_ws, state::ChatState};
use maud::{DOCTYPE, Markup, html};

use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use todos::{
    handlers::{create_todo, delete_todo, get_todos, toggle_todo},
    state::TodosState,
};

pub mod chat;
pub mod todos;
pub mod utils;

pub struct AppState {
    todos: TodosState,
    chat: ChatState,
}
pub type ApiState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        tracing::error!("Error while setting up tracing subscriber: {err:?}");
    };
    tracing::info!("Starting application");

    let state = Arc::new(RwLock::new(AppState {
        todos: TodosState::new(),
        chat: ChatState::new(),
    }));

    let app = Router::new()
        .route("/", get(root))
        .route("/todos", get(get_todos))
        .route("/todo", post(create_todo))
        .route("/todo/{id}/toggle", post(toggle_todo))
        .route("/todo/{id}", delete(delete_todo))
        .route("/chat", get(handle_chat_ws))
        .layer(TraceLayer::new_for_http())
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Could not bind to 0.0.0.0:3001");

    axum::serve(listener, app)
        .await
        .expect("Could not start application");
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
