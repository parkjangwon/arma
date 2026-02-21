use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::config::SharedEngine;
use crate::metrics::{ReasonHit, RuntimeMetrics};

/// Shared API state.
#[derive(Clone)]
pub struct AppState {
    pub engine: SharedEngine,
    pub metrics: std::sync::Arc<RuntimeMetrics>,
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
    pub total_requests: u64,
    pub pass_count: u64,
    pub block_count: u64,
    pub block_rate: f64,
    pub latency_p50_ms: u128,
    pub latency_p95_ms: u128,
    pub top_block_reasons: Vec<ReasonHit>,
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

            state
                .metrics
                .record_validation(validation.is_safe, &validation.reason, latency_ms);

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
            let latency_ms = started.elapsed().as_millis();
            state
                .metrics
                .record_validation(true, "ENGINE_ERROR_BYPASS", latency_ms);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ValidateResponse {
                    is_safe: true,
                    reason: "ENGINE_ERROR_BYPASS".to_string(),
                    score: 0,
                    latency_ms,
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
    let snapshot = state.metrics.snapshot();

    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            filter_pack_version: version,
            total_requests: snapshot.total_requests,
            pass_count: snapshot.pass_count,
            block_count: snapshot.block_count,
            block_rate: snapshot.block_rate,
            latency_p50_ms: snapshot.latency_p50_ms,
            latency_p95_ms: snapshot.latency_p95_ms,
            top_block_reasons: snapshot.top_block_reasons,
        }),
    )
}
