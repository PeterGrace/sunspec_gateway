use crate::consts::*;
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge_vec, CounterVec, Encoder, GaugeVec, TextEncoder,
};

lazy_static! {
    // setup prometheus
    pub static ref STATIC_PROM: PrometheusMetrics = PrometheusMetricsBuilder::new(APP_NAME)
        .endpoint("/metrics")
        .build()
        .unwrap();

    // list o' metrics
    pub static ref APP_INFO: GaugeVec = register_gauge_vec!(
        "sunspec_gateway_app_info",
        "static app labels that potentially only change at restart",
        &["crate_version", "git_hash"]
    ).unwrap();

    pub static ref MQTT_CONFIG_PAYLOADS_SENT: CounterVec = register_counter_vec!(
        "sunspec_gateway_mqtt_config_payloads_sent",
        "Count of mqtt config payloads sent from this process",
        &["serial_number","model","point"]
    ).unwrap();

}

pub fn register_metrics() {
    let reg = &STATIC_PROM.registry;
    reg.register(Box::new(APP_INFO.clone()))
        .expect("Couldn't register metric: {e}");
    reg.register(Box::new(MQTT_CONFIG_PAYLOADS_SENT.clone()))
        .expect("Couldn't register metric: {e}");
}
