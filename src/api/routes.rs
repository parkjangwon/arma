use axum::routing::{get, post};
use axum::Router;

use crate::api::handlers::{health, validate_prompt, AppState};

/// Builds API routes with shared application state.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/validate", post(validate_prompt))
        .route("/health", get(health))
        .with_state(state)
}
