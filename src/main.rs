use std::sync::Arc;

use axum::{
    Form, Json, Router,
    extract::{FromRequest, Path, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use state::AppState;
use templates::todos::{todo_form, todo_view, todos_view};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

pub mod state;
pub mod templates;

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

async fn get_todos(State(state): State<ApiState>, headers: HeaderMap) -> impl IntoResponse {
    let state = state.read().await;

    match headers.get("Accept").and_then(|h| h.to_str().ok()) {
        Some("application/json") => Json(state.todos()).into_response(),
        _ => html! {
            (DOCTYPE)
            html {
                head {
                    script src="/assets/htmx.min.js" {}
                    link href="/assets/style/output.css" rel="stylesheet";
                }
                body.flex.flex-col {
                    h1.text-2xl.text-center.mt-2 { "A basic todos app" }
                    div.self-center.pt-8 {
                        (todo_form())
                    }
                    (todos_view(state.todos()))
                }
            }
        }
        .into_response(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodoRequest {
    content: String,
}
struct ContentNegotiator<T>(T);

impl<S, T> FromRequest<S> for ContentNegotiator<T>
where
    S: Send + Sync,
    T: for<'de> Deserialize<'de> + Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Check content type to determine how to parse
        if content_type.starts_with("application/json") {
            // Parse as JSON
            let Json(payload) = Json::<T>::from_request(req, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            return Ok(ContentNegotiator(payload));
        }

        // Default to form data
        let Form(payload) = Form::<T>::from_request(req, state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(ContentNegotiator(payload))
    }
}

async fn create_todo(
    State(state): State<ApiState>,
    headers: HeaderMap,
    ContentNegotiator(payload): ContentNegotiator<CreateTodoRequest>,
) -> impl IntoResponse {
    let content = payload.content;
    if content.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    let mut state = state.write().await;
    let todo = state.add_todo(&content);

    match headers.get("Accept").and_then(|h| h.to_str().ok()) {
        Some("application/json") => Json(todo).into_response(),
        _ => todo_view(todo).into_response(),
    }
}

async fn toggle_todo(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((id,)): Path<(usize,)>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    let Some(todo) = state.toggle_todo(id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match headers.get("Accept").and_then(|h| h.to_str().ok()) {
        Some("application/json") => Json(todo).into_response(),
        _ => todo_view(todo).into_response(),
    }
}
