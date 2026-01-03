use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::collections::HashMap;

use crate::modules::AppAPIResponse;
use crate::payload::PayloadValueType;
use crate::state::AppState;
use crate::{LIVE_VALUES, MODEL_HASH, SETTINGS};
use sunspec_rs::sunspec_models::{PointIdentifier, ValueType};
use sunspec_rs::sunspec_connection::SunSpecConnection;
use tower_sessions::Session;

const CONTROLS_TAG: &str = "controls";

/// Symbol definition for enum points
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ControlSymbol {
    pub name: String,
    pub value: i64,
}

/// Definition of a controllable point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ControlPointDef {
    pub model_id: u16,
    pub point_name: String,
    pub label: String,
    pub description: String,
    pub data_type: String,
    pub symbols: Option<Vec<ControlSymbol>>,
    pub units: Option<String>,
}

/// Response with current value for a control point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ControlPointState {
    #[serde(flatten)]
    pub def: ControlPointDef,
    pub serial_number: String,
    pub current_value: Option<PayloadValueType>,
}

/// Response containing all controllable points
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct ControlPointsResponse {
    pub points: Vec<ControlPointState>,
}

/// Request to write a value
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WriteRequest {
    pub serial_number: String,
    pub model_id: u16,
    pub point_name: String,
    pub value: String,
}

/// Response from write operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WriteResponse {
    pub success: bool,
    pub message: String,
}

/// Whitelisted control points
fn get_control_point_defs() -> Vec<ControlPointDef> {
    vec![
        ControlPointDef {
            model_id: 64200,
            point_name: "SysMd".to_string(),
            label: "System Operating Mode".to_string(),
            description: "REbus System operational mode".to_string(),
            data_type: "enum16".to_string(),
            symbols: Some(vec![
                ControlSymbol { name: "SAFETY_SHUTDOWN".to_string(), value: 0 },
                ControlSymbol { name: "GRID_CONNECT".to_string(), value: 1 },
                ControlSymbol { name: "SELF_SUPPLY".to_string(), value: 2 },
                ControlSymbol { name: "CLEAN_BACKUP".to_string(), value: 3 },
                ControlSymbol { name: "PRIORITY_BACKUP".to_string(), value: 4 },
                ControlSymbol { name: "ARBITRAGE".to_string(), value: 5 },
                ControlSymbol { name: "FULL_EXPORT".to_string(), value: 6 },
            ]),
            units: None,
        },
        ControlPointDef {
            model_id: 64251,
            point_name: "Ena".to_string(),
            label: "PV Link Enable".to_string(),
            description: "Enable or Disable PV Link".to_string(),
            data_type: "enum16".to_string(),
            symbols: Some(vec![
                ControlSymbol { name: "DISABLE".to_string(), value: 0 },
                ControlSymbol { name: "ENABLE".to_string(), value: 1 },
            ]),
            units: None,
        },
        ControlPointDef {
            model_id: 802,
            point_name: "SocRsvMax".to_string(),
            label: "Max Reserve %".to_string(),
            description: "Maximum reserve for storage as percentage".to_string(),
            data_type: "uint16".to_string(),
            symbols: None,
            units: Some("%WHRtg".to_string()),
        },
        ControlPointDef {
            model_id: 802,
            point_name: "SoCRsvMin".to_string(),
            label: "Min Reserve %".to_string(),
            description: "Minimum reserve for storage as percentage".to_string(),
            data_type: "uint16".to_string(),
            symbols: None,
            units: Some("%WHRtg".to_string()),
        },
        ControlPointDef {
            model_id: 802,
            point_name: "SetOp".to_string(),
            label: "Battery Operation".to_string(),
            description: "Connect or Disconnect battery bank".to_string(),
            data_type: "enum16".to_string(),
            symbols: Some(vec![
                ControlSymbol { name: "CONNECT".to_string(), value: 1 },
                ControlSymbol { name: "DISCONNECT".to_string(), value: 2 },
            ]),
            units: None,
        },
        ControlPointDef {
            model_id: 802,
            point_name: "SetInvState".to_string(),
            label: "Inverter State".to_string(),
            description: "Set the inverter state".to_string(),
            data_type: "enum16".to_string(),
            symbols: Some(vec![
                ControlSymbol { name: "INVERTER_STOPPED".to_string(), value: 1 },
                ControlSymbol { name: "INVERTER_STANDBY".to_string(), value: 2 },
                ControlSymbol { name: "INVERTER_STARTED".to_string(), value: 3 },
            ]),
            units: None,
        },
    ]
}

pub fn controls_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(get_control_points))
        .routes(utoipa_axum::routes!(write_control_point))
}

/// Get all controllable points with current values
#[utoipa::path(
    get,
    path = "/points",
    summary = "Get all controllable points with current values",
    responses(
        (status = OK, description = "List of control points", body = ControlPointsResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Error", body = AppAPIResponse)
    ),
    tag = CONTROLS_TAG
)]
pub async fn get_control_points(
    State(_state): State<AppState>,
    _session: Session,
) -> Result<Json<ControlPointsResponse>, (StatusCode, AppAPIResponse)> {
    let defs = get_control_point_defs();
    let live_values = LIVE_VALUES.read().await;
    
    let mut points = Vec::new();
    
    // For each definition, find matching live values
    for def in defs {
        // Find all serial numbers that have this model
        let mut seen_serials = std::collections::HashSet::new();
        
        for (_key, value) in live_values.iter() {
            if value.model_id == def.model_id {
                if seen_serials.insert(value.serial_number.clone()) {
                    // Look up the specific point value
                    let point_key = format!("{}.{}.{}", value.serial_number, def.model_id, def.point_name);
                    let current_value = live_values.get(&point_key).map(|v| v.value.clone());
                    
                    points.push(ControlPointState {
                        def: def.clone(),
                        serial_number: value.serial_number.clone(),
                        current_value,
                    });
                }
            }
        }
    }
    
    Ok(Json(ControlPointsResponse { points }))
}

/// Write a value to a control point
#[utoipa::path(
    post,
    path = "/write",
    summary = "Write a value to a control point",
    request_body = WriteRequest,
    responses(
        (status = OK, description = "Write result", body = WriteResponse),
        (status = BAD_REQUEST, description = "Invalid request", body = AppAPIResponse),
        (status = INTERNAL_SERVER_ERROR, description = "Error", body = AppAPIResponse)
    ),
    tag = CONTROLS_TAG
)]
pub async fn write_control_point(
    State(_state): State<AppState>,
    _session: Session,
    Json(request): Json<WriteRequest>,
) -> Result<Json<WriteResponse>, (StatusCode, Json<AppAPIResponse>)> {
    let defs = get_control_point_defs();
    
    // Verify point is whitelisted
    let def = defs.iter().find(|d| d.model_id == request.model_id && d.point_name == request.point_name);
    let def = match def {
        Some(d) => d,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AppAPIResponse::message(format!(
                    "Point {}.{} is not a controllable point",
                    request.model_id, request.point_name
                ))),
            ));
        }
    };
    
    // Parse and validate value
    let value: i64 = match request.value.parse() {
        Ok(v) => v,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AppAPIResponse::message("Value must be a valid integer")),
            ));
        }
    };
    
    // For enum types, validate against symbols
    if let Some(symbols) = &def.symbols {
        if !symbols.iter().any(|s| s.value == value) {
            let valid: Vec<_> = symbols.iter().map(|s| format!("{}({})", s.name, s.value)).collect();
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AppAPIResponse::message(format!(
                    "Invalid value {}. Valid options: {}",
                    value,
                    valid.join(", ")
                ))),
            ));
        }
    }
    
    // For uint16, validate range
    if def.data_type == "uint16" {
        if value < 0 || value > 65535 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AppAPIResponse::message("Value must be between 0 and 65535")),
            ));
        }
    }
    
    // Find the device that has both this model AND this serial number
    // The MODEL_HASH is keyed by "addr/slave_id" and contains HashMap<model_id, ModelData>
    let model_hash = MODEL_HASH.read().await;
    let live_values = LIVE_VALUES.read().await;
    
    // First, find which addr/slave combination has this serial number
    let mut device_key: Option<String> = None;
    
    for (key, models) in model_hash.iter() {
        // Check if this device has the model we want
        if models.contains_key(&request.model_id) {
            // Check if any live value for this model has our serial number
            for (_lv_key, lv) in live_values.iter() {
                if lv.serial_number == request.serial_number && lv.model_id == request.model_id {
                    device_key = Some(key.clone());
                    break;
                }
            }
        }
        if device_key.is_some() {
            break;
        }
    }
    
    let key = match device_key {
        Some(k) => k,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AppAPIResponse::message(format!(
                    "Device {} with model {} not found",
                    request.serial_number, request.model_id
                ))),
            ));
        }
    };
    
    // Parse addr and slave_id from key
    let parts: Vec<&str> = key.split('/').collect();
    if parts.len() != 2 {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppAPIResponse::message("Invalid device key format")),
        ));
    }
    let addr = parts[0].to_string();
    let slave_id: u8 = parts[1].parse().unwrap_or(1);
    
    // Get the models for this device
    let models = match model_hash.get(&key) {
        Some(m) => m.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppAPIResponse::message("Model data not found")),
            ));
        }
    };
    drop(model_hash);
    drop(live_values);
    
    let model = match models.get(&request.model_id) {
        Some(m) => m.clone(),
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AppAPIResponse::message(format!(
                    "Model {} not found on device",
                    request.model_id
                ))),
            ));
        }
    };
    
    // Get TLS config if available
    let settings = SETTINGS.read().await;
    let tls_config = settings.units.iter()
        .find(|u| u.addr == addr)
        .and_then(|u| u.tls.clone());
    drop(settings);
    
    // Create a new connection to write the value
    let mut conn = match SunSpecConnection::new(addr.clone(), Some(slave_id), false, tls_config).await {
        Ok(c) => c,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppAPIResponse::message(format!("Failed to connect: {}", e))),
            ));
        }
    };
    
    // Set models on connection
    conn.models = models;
    
    // Perform the write
    match conn
        .set_point(
            model,
            PointIdentifier::Point(request.point_name.clone()),
            ValueType::Integer(value),
        )
        .await
    {
        Ok(_) => {
            tracing::info!(
                "Successfully wrote {}={} to {}.{}",
                request.point_name,
                value,
                request.serial_number,
                request.model_id
            );
            Ok(Json(WriteResponse {
                success: true,
                message: format!("Successfully set {} to {}", request.point_name, value),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to write point: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppAPIResponse::message(format!("Write failed: {}", e))),
            ))
        }
    }
}
