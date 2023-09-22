use crate::ipc::{HAConfigPayload, PayloadValueType, StatePayload};
use crate::monitored_point::MonitoredPoint;
use crate::sunspec_unit::SunSpecUnit;
use sunspec_rs::sunspec_models::{Point, ValueType};

const DEFAULT_DISPLAY_PRECISION: Option<u8> = Some(4_u8);

#[instrument]
pub fn generate_payloads(
    unit: &SunSpecUnit,
    point_data: &Point,
    monitored_point: &MonitoredPoint,
    val: &ValueType,
) -> (HAConfigPayload, StatePayload) {
    let sn = unit.serial_number.clone();
    let model = monitored_point.model.clone();
    let point_name = monitored_point.name.clone();
    let mut config_payload: HAConfigPayload = HAConfigPayload::default();
    let mut state_payload: StatePayload = StatePayload::default();
    let state_topic = format!("sunspec_gateway/{sn}/{model}/{point_name}");
    config_payload.name = format!("{sn}: {model}-{point_name}");
    config_payload.state_topic = state_topic.clone();
    config_payload.device_class = monitored_point.device_class.clone();
    config_payload.state_class = monitored_point.state_class.clone();
    config_payload.expires_after = 300;
    config_payload.value_template = Some("{{ value_json.value }}".to_string());
    config_payload.unique_id = format!("{sn}.{model}.{point_name}");
    config_payload.entity_id = format!("sensor.{sn}_{model}_{point_name}");
    match val {
        ValueType::String(str) => {
            debug!("Response for {model}/{point_name}: {str}");
            state_payload.value = PayloadValueType::String(str.to_owned())
        }
        ValueType::Integer(int) => {
            debug!("Response for {model}/{point_name}: {int}");
            if monitored_point.uom.is_some() {
                // we are overriding the default uom from config
                config_payload.native_uom = monitored_point.uom.clone();
            } else {
                config_payload.native_uom = point_data.units.clone();
            }
            state_payload.value = PayloadValueType::Int(*int as i64)
        }
        ValueType::Float(float) => {
            debug!("Response for {model}/{point_name}: {0:.1}", float);
            if monitored_point.precision.is_some() {
                config_payload.suggested_display_precision = monitored_point.precision;
            } else {
                config_payload.suggested_display_precision = DEFAULT_DISPLAY_PRECISION;
            };
            if monitored_point.uom.is_some() {
                // we are overriding the default uom from config
                config_payload.native_uom = monitored_point.uom.clone();
            } else {
                config_payload.native_uom = point_data.units.clone();
            }
            state_payload.value = PayloadValueType::Float(*float as f64);
            if let Some(literal) = &point_data.literal {
                if literal.label.is_some() {
                    config_payload.name = literal.label.clone().unwrap();
                }
                state_payload.label = literal.clone().label;
                state_payload.description = literal.clone().description;
                state_payload.notes = literal.clone().notes;
            }
        }
        ValueType::Boolean(boolean) => {
            debug!("Response for {model}/{point_name}: {boolean}");
            state_payload.value = PayloadValueType::String(boolean.to_string());
        }
        ValueType::Array(vec) => {
            debug!("Response for {model}/{point_name}: {:#?}", vec);
            let concat = vec.join(", ");
            state_payload.value = PayloadValueType::String(concat)
        }
    }
    config_payload.device = unit.device_info.clone();

    (config_payload, state_payload)
}
