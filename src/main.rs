#[macro_use]
extern crate tracing;
extern crate strum;
#[macro_use]
extern crate thiserror;

mod cli_args;
mod config_structs;
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
use crate::config_structs::GatewayConfig;
use crate::config_structs::TracingConfig;
use crate::ipc::{IPCMessage, InboundMessage, PublishMessage};
use crate::mqtt_connection::MqttConnection;
use crate::mqtt_poll::mqtt_poll_loop;
use crate::state_mgmt::prepare_to_database;
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
use sqlx::{Pool, Sqlite};
use std::collections::VecDeque;
use std::fs;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use sunspec_unit::SunSpecUnit;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};
use tracing_log::AsTrace;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{filter, prelude::*, EnvFilter, Layer, Registry};

const MPSC_BUFFER_SIZE: usize = 100_usize;

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
    static ref SETTINGS: RwLock<GatewayConfig> = RwLock::new({
         let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
             Ok(s) => s,
             Err(_e) => { "./config.yaml".to_string()}
         };
        let yaml = fs::read_to_string(cfg_file).unwrap_or_else(|e| {
            die(&format!("Can't read config file: {e}"));
            String::default()
            });
        let gc: GatewayConfig = match serde_yaml::from_str(&yaml)  {
            Ok(gc) => gc,
            Err(e) => { die(&format!("Couldn't deserialize GatewayConfig: {e}"));
            GatewayConfig::default()}
        };
        gc
    });
    //endregion
}

pub fn die(msg: &str) {
    println!("{}", msg);
    process::exit(1);
}

#[tokio::main]
async fn main() {
    //region initialize app and logging
    // disabling clap for the moment while I decide what I want to do with this vs. envvars
    //let cli = CliArgs::parse();
    let (tx, mut rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (mqtt_tx, mut mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (from_mqtt_tx, mut from_mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (broadcast_tx, mut broadcast_rx) = broadcast::channel::<IPCMessage>(16_usize);

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

    let config = SETTINGS.read().await;
    let tracer_layer = if config.tracing.is_some() {
        let t = config.tracing.clone().unwrap();
        Some(tracing_opentelemetry::layer().with_tracer(make_tracer(t.url, t.sample_rate)))
    } else {
        None
    };

    let subscriber = subscriber.with(tracer_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Can't set global subscriber for logging.");
    global::set_text_map_propagator(TraceContextPropagator::new());

    //endregion

    //region databasey stuff
    if let Err(e) = prepare_to_database().await {
        die(&format!("Can't database: {e}"))
    }
    //endregion

    //region create mqtt server connection and spawn mqtt thread
    let mqtt_conn = match MqttConnection::new(
        config
            .mqtt_client_id
            .clone()
            .unwrap_or("sunspec_gateway".to_string()),
        config.mqtt_server_addr.clone(),
        config.mqtt_server_port.unwrap_or(1883),
        config.mqtt_username.clone(),
        config.mqtt_password.clone(),
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            return die("Couldn't create mqtt connection object: {e}");
        }
    };
    let mqtt_handler = tokio::task::spawn(async move {
        mqtt_poll_loop(mqtt_conn, mqtt_rx, from_mqtt_tx).await;
    });
    //endregion

    //region populate sunspec devices into an array
    let units = config.units.clone();
    let mut devices: Vec<SunSpecUnit> = vec![];
    for u in units {
        for s in u.slaves.iter() {
            match SunSpecUnit::new(u.addr.clone(), s.to_string()).await {
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
        let bcast_rx = broadcast_tx.subscribe();
        tasks.spawn(async move {
            match poll_loop(&d, tx, bcast_rx).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });
    }
    // drop write mutex on tasks

    //endregion

    //region watch the mpsc tasks receive loop
    let mut msg_queue: VecDeque<PublishMessage> = VecDeque::new();
    let mut incoming_control_queue: VecDeque<InboundMessage> = VecDeque::new();
    loop {
        //endregion
        //region sunspec device channel loop handling
        match rx.try_recv() {
            Ok(ipcm) => match ipcm {
                IPCMessage::Outbound(o) => {
                    msg_queue.push_front(o);
                }
                IPCMessage::Error(e) => {
                    die(&format!("serial_number={}: {}", e.serial_number, e.msg));
                }
                IPCMessage::PleaseReconnect(addr, slave) => {
                    let tx = tx.clone();
                    let bcast_rx = broadcast_tx.subscribe();
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
                        match poll_loop(&ssu, tx, bcast_rx).await {
                            Ok(_) => Ok(()),
                            Err(e) => return Err(e),
                        }
                    });
                }
                IPCMessage::Inbound(_) => {
                    // we don't send inbounds to mqtt
                    unreachable!();
                }
            },
            Err(_) => {}
        };

        match from_mqtt_rx.try_recv() {
            Ok(recvd) => match recvd {
                IPCMessage::Inbound(inmsg) => {
                    info!(
                        "Received payload for {},{},{}:{}",
                        inmsg.serial_number, inmsg.model, inmsg.point_name, inmsg.payload
                    );
                    incoming_control_queue.push_front(inmsg.clone());
                }
                IPCMessage::Outbound(_) => {
                    unreachable!();
                }
                IPCMessage::PleaseReconnect(_, _) => {
                    unreachable!();
                }
                IPCMessage::Error(_) => {
                    unreachable!();
                }
            },
            Err(_) => {}
        };

        //endregion
        while let Some(msg) = incoming_control_queue.pop_front() {
            if let Err(e) = broadcast_tx.send(IPCMessage::Inbound(msg)) {
                error!("Unable to broadcast message to threads: {e}");
            }
        }

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

        let _ = sleep(Duration::from_millis(100));
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
