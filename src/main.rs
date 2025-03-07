use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use maud::{DOCTYPE, Markup, html};

use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use todos::{
    handlers::{create_todo, delete_todo, get_todos, toggle_todo},
    state::AppState,
};

pub mod todos;
pub mod utils;

type ApiState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(RwLock::new(AppState::new()));

    let app = Router::new()
        .route("/", get(root))
        .route("/todos", get(get_todos))
        .route("/todo", post(create_todo))
        .route("/todo/{id}/toggle", post(toggle_todo))
        .route("/todo/{id}", delete(delete_todo))
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
                link href="/assets/style/output.css" rel="stylesheet";
            }
            body {
                h1.text-2xl.font-bold.text-center.mt-2 { "htmx + rust" }
                div.flex.flex-col.mt-4 {
                    a.link.self-center href="/todos" { "Todos" }
                }
            }
        }
    )
}
