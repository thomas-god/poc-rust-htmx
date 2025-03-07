use axum::{
    Form, Json,
    extract::{FromRequest, Request},
    http::{StatusCode, header},
};
use serde::Deserialize;

pub struct ContentNegotiator<T>(pub T);

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
