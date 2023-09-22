#[macro_use]
extern crate tracing;
extern crate strum;
#[macro_use]
extern crate thiserror;

mod cli_args;
mod date_serializer;
mod ipc;
mod monitored_point;
mod mqtt_connection;
mod mqtt_poll;
mod payload;
mod state_mgmt;
mod sunspec_poll;
mod sunspec_unit;

use crate::cli_args::CliArgs;
use crate::ipc::{IPCMessage, Payload, PublishMessage};
use crate::mqtt_connection::MqttConnection;
use crate::mqtt_poll::mqtt_poll_loop;
use crate::sunspec_poll::poll_loop;
use clap::Parser;
use config::Config;
use console_subscriber;
use futures::{FutureExt, TryFutureExt};
use lazy_static::lazy_static;
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use serde::Deserialize;
use std::collections::VecDeque;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use sunspec_unit::SunSpecUnit;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};
use tracing_log::AsTrace;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{prelude::*, EnvFilter, Layer, Registry};

const MPSC_BUFFER_SIZE: usize = 100_usize;

#[derive(Deserialize)]
pub struct TracingConfig {
    url: String,
    sample_rate: f32,
}

#[derive(Error, Debug, Default)]
pub enum GatewayError {
    #[error("Communication Error: {0}")]
    CommunicationError(String),
    #[error("Error from thread: {0}")]
    Error(String),
    #[error("Unspecified error")]
    #[default]
    Unspecified,
}

lazy_static! {
    static ref TASK_PILE: RwLock<JoinSet<Result<(),GatewayError>>> = RwLock::new(JoinSet::<Result<(),GatewayError>>::new());
    //region create SETTINGS static object
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
            Ok(s) => s,
            Err(_e) => { "./config.yaml".to_string()}
        };
        let settings = match Config::builder()
            .add_source(config::File::with_name(&cfg_file))
            .add_source(
                config::Environment::with_prefix("SUNSPEC_GATEWAY")
                .try_parsing(true)
                .list_separator(",")
            )
            .build()
            {
                Ok(s) => s,
                Err(e) => {
                    error!("{}", e);
                    process::exit(1);
                }
            };
        settings
    });
    //endregion
}

pub fn die(msg: &str) {
    error!(msg);
    process::exit(1);
}

#[tokio::main]
async fn main() {
    //region initialize app and logging
    //let cli = CliArgs::parse();
    let config = SETTINGS.read().await;

    let tracing_config: Option<TracingConfig> = config.get::<TracingConfig>("tracing").ok();
    let (tx, mut rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (mqtt_tx, mut mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);

    global::set_text_map_propagator(TraceContextPropagator::new());
    let console_layer = console_subscriber::spawn();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO"));
    let format_layer = tracing_subscriber::fmt::layer().event_format(
        tracing_subscriber::fmt::format()
            .with_file(true)
            .with_line_number(true),
    );
    let mut subscriber = Registry::default()
        .with(console_layer)
        .with(env_filter)
        .with(format_layer);

    let tracer_layer = if tracing_config.is_some() {
        let t = tracing_config.unwrap();
        Some(tracing_opentelemetry::layer().with_tracer(make_tracer(t.url, t.sample_rate)))
    } else {
        None
    };

    let subscriber = subscriber.with(tracer_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Can't set global subscriber for logging.");

    //endregion

    //region get config and load in sunspec unit defs
    let units = match config.get_array("units") {
        Ok(u) => u,
        Err(e) => {
            error!("unable to get unit definitions from config file: {e}");
            process::exit(1);
        }
    };
    //endregion

    //region create mqtt server connection and spawn mqtt thread
    let mqtt_conn = match MqttConnection::new(
        config
            .get_string("mqtt_client_id")
            .unwrap_or("sunspec_gateway".to_string()),
        config.get_string("mqtt_server_addr").unwrap_or_else(|_| {
            die("mqtt_server_addr not defined");
            String::default()
        }),
        config.get_int("mqtt_port").unwrap_or(1883) as u16,
        config.get_string("mqtt_username").ok(),
        config.get_string("mqtt_password").ok(),
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            return die("Couldn't create mqtt connection object: {e}");
        }
    };
    let mqtt_handler = tokio::task::spawn(async move {
        mqtt_poll_loop(mqtt_conn, mqtt_rx).await;
    });
    //endregion

    //region populate sunspec devices into an array
    let mut devices: Vec<SunSpecUnit> = vec![];
    for u in units {
        let table = u.clone().into_table().unwrap();
        let v_slaves = table
            .clone()
            .get("slaves")
            .unwrap()
            .clone()
            .into_array()
            .unwrap();
        for s in v_slaves.iter() {
            let slave = s.clone().into_uint().unwrap();
            match SunSpecUnit::new(
                table
                    .clone()
                    .get("addr")
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap(),
                slave.to_string(),
            )
            .await
            {
                Ok(p) => devices.push(p),
                Err(e) => {
                    die(&format!("Unable to create connection to SunSpec Unit: {e}"));
                }
            };
        }
    }
    //endregion

    //region create sunspec thread workers
    for d in devices {
        let mut tasks = TASK_PILE.write().await;
        let tx = tx.clone();
        tasks.spawn(async move {
            match poll_loop(&d, tx).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });
    }
    // drop write mutex on tasks

    //endregion

    //region watch the mpsc tasks receive loop
    let mut msg_queue: VecDeque<PublishMessage> = VecDeque::new();
    loop {
        //endregion
        //region sunspec device channel loop handling
        match rx.recv().await {
            Some(ipcm) => match ipcm {
                IPCMessage::Outbound(o) => {
                    msg_queue.push_front(o);
                }
                IPCMessage::Error(e) => {
                    die(&format!("serial_number={}: {}", e.serial_number, e.msg));
                }
                IPCMessage::PleaseReconnect(addr, slave) => {
                    let tx = tx.clone();
                    let ssu = match SunSpecUnit::new(addr.clone(), slave.to_string()).await {
                        Ok(s) => s,
                        Err(e) => {
                            return die(&format!(
                                "Couldn't create new sunspecunit to replace dead conn: {e}"
                            ));
                        }
                    };
                    info!("We received a PleaseReconnect message for {addr}/{slave}");
                    let mut tasks = TASK_PILE.write().await;
                    tasks.spawn(async move {
                        match poll_loop(&ssu, tx).await {
                            Ok(_) => Ok(()),
                            Err(e) => return Err(e),
                        }
                    });
                }
            },
            None => {
                error!("Error receiving ipc message, None returned on rx.recv().await")
            }
        }

        //endregion

        while let Some(msg) = msg_queue.pop_front() {
            match timeout(
                Duration::from_secs(10),
                mqtt_tx.send(IPCMessage::Outbound(msg)),
            )
            .await
            {
                Ok(future) => {
                    if let Err(e) = future {
                        error!("Unable to send mqtt mpsc tx message: {e}");
                    }
                }
                Err(e) => {
                    debug!("Timeout sending a mpsc transmission?: {e}");
                }
            }
        }

        // check cleanups

        let mut tasks = TASK_PILE.write().await;
        match tasks.join_next().now_or_never() {
            Some(task) => {
                match task {
                    Some(t) => {
                        match t {
                            Ok(t1) => match t1 {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("{e}");
                                }
                            },
                            Err(e) => {
                                // TODO: what does Err mean here?
                                error!("Got an error when checking joinset: {e}");
                            }
                        }
                    }
                    None => {
                        // TODO: what does none mean here?
                    }
                }
            }
            None => {
                // no tasks waiting to report in
            }
        }

        if mqtt_handler.is_finished() {
            die("MQTT thread exited.");
        }

        let _ = sleep(Duration::from_millis(1000));
    }
    //endregion
}

pub fn make_tracer(url: String, sample: f32) -> Tracer {
    let exporter = opentelemetry_otlp::new_exporter().http().with_endpoint(url);
    let otlp_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    "sunspec_gateway",
                )]))
                .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(1.0)),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Can't create tracer");
    otlp_tracer
}
