use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use shielder_relayer::SimpleServiceResponse;
use tracing::{debug, error};

use crate::{monitor::healthy, AppState};

pub async fn health_endpoint(app_state: State<AppState>) -> impl IntoResponse {
    debug!("Healthcheck request received");
    match healthy(&app_state.node_rpc_url).await {
        Ok(()) => (StatusCode::OK, SimpleServiceResponse::from("Healthy")),
        Err(err) => service_unavailable(&err),
    }
}

fn service_unavailable(msg: &str) -> (StatusCode, Json<SimpleServiceResponse>) {
    error!(msg);
    (
        StatusCode::SERVICE_UNAVAILABLE,
        SimpleServiceResponse::from(msg),
    )
}
