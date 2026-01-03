use crate::config_structs::GatewayConfig;
use crate::state::AppState;
use crate::state_mgmt::{load_config, save_config};
use crate::SETTINGS;
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::get;
use utoipa_axum::{routes, router::OpenApiRouter};
use crate::modules::status_structs::SystemStatus;

pub fn settings_routes(state: AppState) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_config, update_config))
        .routes(routes!(restart_server))
        .routes(routes!(get_status))
        .with_state(state)
}

/// Get current configuration
#[utoipa::path(
    get,
    path = "",
    tag = "settings",
    responses(
        (status = 200, description = "Current configuration", body = GatewayConfig)
    )
)]
pub async fn get_config() -> Json<GatewayConfig> {
    let config = SETTINGS.read().await;
    Json(config.clone())
}

/// Update configuration
#[utoipa::path(
    post,
    path = "",
    tag = "settings",
    request_body = GatewayConfig,
    responses(
        (status = 200, description = "Configuration updated", body = GatewayConfig)
    )
)]
pub async fn update_config(
    State(_state): State<AppState>,
    Json(payload): Json<GatewayConfig>,
) -> Json<GatewayConfig> {
    // Save to DB
    if let Err(e) = save_config(&payload).await {
        error!("Failed to save config to DB: {e}");
    }
    
    // Update memory
    let mut config = SETTINGS.write().await;
    *config = payload.clone();
    
    // Note: Some changes (like MQTT) might require a restart to take effect fully
    // as we don't hot-reload the MQTT client or SunSpec connections yet.
    
    Json(payload)
}

/// Restart the server
#[utoipa::path(
    post,
    path = "/restart",
    tag = "settings",
    responses(
        (status = 200, description = "Server restarting")
    )
)]
pub async fn restart_server() -> Json<serde_json::Value> {
    info!("Restart requested via API");
    // Spawn a thread to exit after a short delay to allow response to send
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        std::process::exit(0);
    });
    Json(serde_json::json!({"status": "restarting"}))
}

/// Get system status
#[utoipa::path(
    get,
    path = "/status",
    tag = "settings",
    responses(
        (status = 200, description = "System Status", body = SystemStatus)
    )
)]
pub async fn get_status(
    State(state): State<AppState>,
) -> Json<crate::modules::status_structs::SystemStatus> {
    let status = state.status.read().await;
    Json(status.clone())
}
