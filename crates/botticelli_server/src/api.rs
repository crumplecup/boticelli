//! HTTP API for bot server metrics and health checks.
//!
//! Available with the `metrics` feature.

#[cfg(feature = "metrics")]
use crate::metrics::MetricsCollector;
#[cfg(feature = "metrics")]
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
#[cfg(feature = "metrics")]
use serde_json::json;
#[cfg(feature = "metrics")]
use std::sync::Arc;
#[cfg(feature = "metrics")]
use tracing::instrument;

/// API server state.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct ApiState {
    /// Metrics collector.
    pub metrics: Arc<MetricsCollector>,
}

#[cfg(feature = "metrics")]
impl ApiState {
    /// Creates a new API state.
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }
}

/// Creates the API router.
#[cfg(feature = "metrics")]
pub fn create_router(metrics: Arc<MetricsCollector>) -> Router {
    let state = ApiState { metrics };

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/metrics/bots", get(get_bot_metrics))
        .route("/metrics/narratives", get(get_narrative_metrics))
        .with_state(state)
}

/// Health check endpoint.
#[cfg(feature = "metrics")]
#[instrument(skip_all)]
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

/// Get all metrics in JSON format.
#[cfg(feature = "metrics")]
#[instrument(skip(state))]
async fn get_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let snapshot = state.metrics.snapshot();
    (StatusCode::OK, Json(snapshot))
}

/// Get bot-specific metrics.
#[cfg(feature = "metrics")]
#[instrument(skip(state))]
async fn get_bot_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let snapshot = state.metrics.snapshot();
    (StatusCode::OK, Json(json!({ "bots": snapshot.bots() })))
}

/// Get narrative execution metrics.
#[cfg(feature = "metrics")]
#[instrument(skip(state))]
async fn get_narrative_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let snapshot = state.metrics.snapshot();
    (
        StatusCode::OK,
        Json(json!({ "narratives": snapshot.narratives() })),
    )
}
