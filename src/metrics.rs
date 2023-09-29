use crate::consts::*;
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use lazy_static::lazy_static;
use prometheus::{
    histogram_opts, opts, register_counter_vec, register_gauge_vec, CounterVec, GaugeVec,
    HistogramOpts,
};
use sunspec_rs::metrics::{MODBUS_GET, MODBUS_SET};

const PROM_NAMESPACE: &str = "sunspec_gateway";

macro_rules! app_opts {
    ($a:expr, $b:expr) => {
        opts!($a, $b).namespace(PROM_NAMESPACE)
    };
}
macro_rules! app_histogram_opts {
    ($a:expr, $b:expr, $c:expr) => {
        histogram_opts!($a, $b, $c).namespace(PROM_NAMESPACE)
    };
}

lazy_static! {
    // setup prometheus
    pub static ref STATIC_PROM: PrometheusMetrics = PrometheusMetricsBuilder::new(APP_NAME)
        .endpoint("/metrics")
        .build()
        .unwrap();

    // list o' metrics
    pub static ref APP_INFO: GaugeVec = register_gauge_vec!(
        app_opts!(
        "app_info",
        "static app labels that potentially only change at restart"),
        &["crate_version", "git_hash"]
    ).unwrap();

    pub static ref MQTT_CONFIG_PAYLOADS_SENT: CounterVec = register_counter_vec!(
        app_opts!(
        "mqtt_config_payloads_sent",
        "Count of mqtt config payloads sent from this process"),
        &["serial_number","model","point"]
    ).unwrap();

}

pub fn register_metrics() {
    let reg = &STATIC_PROM.registry;
    reg.register(Box::new(APP_INFO.clone()))
        .expect("Couldn't register metric: {e}");
    reg.register(Box::new(MQTT_CONFIG_PAYLOADS_SENT.clone()))
        .expect("Couldn't register metric: {e}");
    reg.register(Box::new(MODBUS_GET.clone()))
        .expect("Couldn't register metric: {e}");
    reg.register(Box::new(MODBUS_SET.clone()))
        .expect("Couldn't register metric: {e}");
}
