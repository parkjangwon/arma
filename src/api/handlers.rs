use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::config::SharedEngine;

/// Shared API state.
#[derive(Clone)]
pub struct AppState {
    pub engine: SharedEngine,
}

/// Prompt validation request payload.
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub prompt: String,
    pub user_id: Option<String>,
}

/// Prompt validation response payload.
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub is_safe: bool,
    pub reason: String,
    pub score: u32,
    pub latency_ms: u128,
}

/// Health response payload.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub filter_pack_version: String,
}

/// Handles prompt safety validation.
pub async fn validate_prompt(
    State(state): State<AppState>,
    Json(payload): Json<ValidateRequest>,
) -> impl IntoResponse {
    let started = Instant::now();

    let result = {
        let guard = state.engine.read().await;
        guard.validate(&payload.prompt)
    };

    match result {
        Ok(validation) => {
            let latency_ms = started.elapsed().as_millis();
            let action = if validation.is_safe { "PASS" } else { "BLOCK" };
            let matched_keyword = extract_matched_keyword(&validation.reason);

            tracing::info!(
                action = %action,
                matched_keyword = %matched_keyword,
                reason = %validation.reason,
                score = validation.score,
                latency_ms = latency_ms,
                user_id = payload.user_id.as_deref().unwrap_or("anonymous"),
                "prompt validation completed"
            );

            (
                StatusCode::OK,
                Json(ValidateResponse {
                    is_safe: validation.is_safe,
                    reason: validation.reason,
                    score: validation.score,
                    latency_ms,
                }),
            )
                .into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "prompt validation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ValidateResponse {
                    is_safe: true,
                    reason: "ENGINE_ERROR_BYPASS".to_string(),
                    score: 0,
                    latency_ms: started.elapsed().as_millis(),
                }),
            )
                .into_response()
        }
    }
}

fn extract_matched_keyword(reason: &str) -> &str {
    if let Some((_, keyword)) = reason.split_once(':') {
        return keyword;
    }

    match reason {
        "BLOCK_DENY_PATTERN" => "regex_pattern",
        "BYPASS_ALLOW_KEYWORD" => "allow_keyword",
        _ => "none",
    }
}

/// Returns service health and loaded filter-pack metadata.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let version = {
        let guard = state.engine.read().await;
        guard.filter_pack_version().to_string()
    };

    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            filter_pack_version: version,
        }),
    )
}
