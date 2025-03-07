use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use maud::{DOCTYPE, html};
use serde::Deserialize;

use crate::{
    ApiState,
    todos::templates::{todo_form, todos_view},
    utils::ContentNegotiator,
};

use super::templates::todo_view;

pub async fn get_todos(State(state): State<ApiState>, headers: HeaderMap) -> impl IntoResponse {
    let state = state.read().await;

    match headers.get("Accept").and_then(|h| h.to_str().ok()) {
        Some("application/json") => Json(state.todos.todos()).into_response(),
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
                    (todos_view(state.todos.todos()))
                }
            }
        }
        .into_response(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodoRequest {
    pub content: String,
}

pub async fn create_todo(
    State(state): State<ApiState>,
    headers: HeaderMap,
    ContentNegotiator(payload): ContentNegotiator<CreateTodoRequest>,
) -> impl IntoResponse {
    let content = payload.content;
    if content.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    let mut state = state.write().await;
    let todo = state.todos.add_todo(&content);

    match headers.get("Accept").and_then(|h| h.to_str().ok()) {
        Some("application/json") => Json(todo).into_response(),
        _ => todo_view(todo).into_response(),
    }
}

pub async fn toggle_todo(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((id,)): Path<(usize,)>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    let Some(todo) = state.todos.toggle_todo(id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match headers.get("Accept").and_then(|h| h.to_str().ok()) {
        Some("application/json") => Json(todo).into_response(),
        _ => todo_view(todo).into_response(),
    }
}

pub async fn delete_todo(State(state): State<ApiState>, Path((id,)): Path<(usize,)>) -> StatusCode {
    let mut state = state.write().await;
    let Some(_todo) = state.todos.delete_todo(id) else {
        return StatusCode::NOT_FOUND;
    };
    StatusCode::OK
}
