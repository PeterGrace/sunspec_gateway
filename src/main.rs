#[macro_use]
extern crate tracing;
extern crate strum;
#[macro_use]
extern crate thiserror;

mod auth;
mod cli_args;
mod config_structs;
mod consts;
mod date_serializer;
mod ipc;
mod modules;
mod monitored_point;
mod mqtt_connection;
mod mqtt_poll;
mod payload;
mod routes;
mod state;
mod state_mgmt;
mod sunspec_poll;
mod sunspec_unit;

use crate::auth::token_middleware::auth_middleware;
use crate::config_structs::GatewayConfig;
use crate::routes::USERS_TAG;
use axum::middleware;
use std::net::Ipv6Addr;
use tokio::net::TcpListener;
use tower_sessions::Expiry;
use tower_sessions::MemoryStore;
use tower_sessions::SessionManagerLayer;
use utoipa_axum::router::OpenApiRouter;

use crate::consts::*;
use crate::ipc::{IPCMessage, InboundMessage, PublishMessage};
use crate::modules::points::point_routes;
use crate::mqtt_connection::MqttConnection;
use crate::mqtt_poll::mqtt_poll_loop;
use crate::state_mgmt::prepare_to_database;
use crate::sunspec_poll::poll_loop;

use console_subscriber as tokio_console_subscriber;
use futures::FutureExt;
use lazy_static::lazy_static;
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::{BatchConfig, Tracer};
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::process;

use crate::auth::token_extractor::JwksCache;
use crate::modules::users::user_routes;
use crate::routes::{openapi, register_routes, ApiDoc};
use crate::state::AppState;
use axum::http::Method;
use axum::routing::get;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use std::time::Duration;
use sunspec_rs::model_data::ModelData;
use sunspec_rs::sunspec_connection::TlsConfig;
use sunspec_unit::SunSpecUnit;
use tokio::sync::{broadcast, mpsc, OnceCell, RwLock};
use tokio::task;
use tokio::time::{sleep, timeout, Instant};
use tower_http::cors::{Any, CorsLayer};
use tower_sessions::cookie::time::Duration as CookieDuration;
use tracing::Instrument;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[derive(Error, Debug, Default)]
pub enum GatewayError {
    #[error("Communication Error: {0}")]
    CommunicationError(String),
    #[error("Error from thread: {0}")]
    Error(String),
    #[error("Exiting thread")]
    ExitingThread,
    #[error("Unspecified error")]
    #[default]
    Unspecified,
}

lazy_static! {
    static ref SHUTDOWN: OnceCell<bool> = OnceCell::new();
    static ref TASK_PILE: RwLock<task::JoinSet<Result<(),GatewayError>>> = RwLock::new(task::JoinSet::<Result<(),GatewayError>>::new());
    pub static ref API_DOC: OnceCell<utoipa::openapi::OpenApi> = OnceCell::new();

    static ref MODEL_HASH: RwLock<HashMap<String, HashMap<u16, ModelData>>> = RwLock::new(HashMap::new());

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
#[instrument]
async fn main() {
    //region initialize app and logging
    // disabling clap for the moment while I decide what I want to do with this vs. envvars
    //let cli = CliArgs::parse();
    std::panic::set_hook(Box::new(|panic_info| {
        error!("Thread panicked: {}", panic_info);
        //die("thread panic");
    }));

    debug!(
        "sunspec_gateway cargo:{}, githash:{}",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    );

    let (tx, mut rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (mqtt_tx, mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (from_mqtt_tx, mut from_mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (broadcast_tx, _broadcast_rx) = broadcast::channel::<IPCMessage>(16_usize);

    let bcasttx = broadcast_tx.clone();
    let _ = ctrlc::set_handler(move || {
        println!("Received Ctrl-C, communicating to threads to stop");
        //opentelemetry::global::shutdown_tracer_provider();
        let _ = SHUTDOWN.set(true);
        let _ = bcasttx.send(IPCMessage::Shutdown);
    });

    let console_layer = tokio_console_subscriber::spawn();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO"));
    let format_layer = tracing_subscriber::fmt::layer()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_span_events(FmtSpan::NONE);

    let subscriber = Registry::default()
        .with(console_layer)
        .with(env_filter)
        .with(format_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Can't set global subscriber for logging.");

    let mut tracer: Option<Tracer> = None;
    let config = SETTINGS.read().await;
    // let tracer_layer = if config.tracing.is_some() {
    //     let t = config.tracing.clone().unwrap();
    //     let tracer = Some(make_tracer(t.url, t.sample_rate));
    //     Some(tracing_opentelemetry::layer().with_tracer(tracer.unwrap().clone()))
    // } else {
    //     None
    // };
    // let subscriber = subscriber.with(tracer_layer);
    // tracing::subscriber::set_global_default(subscriber)
    //     .expect("Can't set global subscriber for logging.");
    // global::set_text_map_propagator(TraceContextPropagator::new());

    //endregion

    //region databasey stuff
    if let Err(e) = prepare_to_database().await {
        die(&format!("Can't database: {e}"))
    }
    //endregion

    let user_cache = None;
    let state = AppState {
        jwks_cache: JwksCache::new(),
        user_cache,
    };

    //region axum route setup and serve()
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(CookieDuration::hours(
            SESSION_INACTIVITY_LIMIT_HOURS,
        )));

    let cors_layer = CorsLayer::new()
        .allow_methods([
            Method::OPTIONS,
            Method::GET,
            Method::PUT,
            Method::POST,
            Method::DELETE,
        ])
        .allow_origin(Any)
        .allow_headers(Any);

    let auth_layer = middleware::from_fn_with_state(state.clone(), auth_middleware);

    let api = ApiDoc::openapi();

    let public_routes = OpenApiRouter::new()
        .merge(register_routes(state.clone()))
        .nest(
            &format!("{API_VER}/{POINTS_TAG}"),
            point_routes(state.clone()),
        )
        .route(API_PATH, get(openapi));

    let protected_routes = OpenApiRouter::<AppState>::new()
        .nest(
            &format!("{API_VER}/{USERS_TAG}"),
            user_routes(state.clone()),
        )
        .layer(auth_layer);

    let (router, api) = OpenApiRouter::with_openapi(api)
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors_layer)
        .layer(session_layer)
        .with_state(state)
        .split_for_parts();

    let app = router.merge(Scalar::with_url(SCALAR_PATH, api.clone()));
    if let Err(e) = API_DOC.set(api) {
        error!("Couldn't store api into document variable: {e}");
    }

    let listener = TcpListener::bind((Ipv6Addr::UNSPECIFIED, 8080))
        .await
        .expect("Failed to bind");
    let _ = tokio::task::Builder::new()
        .name("axum-listener")
        .spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

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
        Err(_e) => {
            return die("Couldn't create mqtt connection object: {e}");
        }
    };
    let bcasttx = broadcast_tx.clone();
    let mqtt_handler = tokio::task::Builder::new()
        .name("mqtt_thread")
        .spawn(async move {
            let _ = mqtt_poll_loop(
                mqtt_conn,
                mqtt_rx,
                bcasttx.clone().subscribe(),
                from_mqtt_tx,
            )
            .await;
        })
        .unwrap();
    //endregion
    let mut retry_queue: VecDeque<(String, u8, DateTime<Utc>)> = VecDeque::new();
    //region populate sunspec devices into an array
    let units = config.units.clone();
    let mut devices: Vec<SunSpecUnit> = vec![];
    for u in units {
        for s in u.slaves.iter() {
            let addr = u.addr.clone();
            let slave = s.clone().to_string();
            info!("connecting to unit {addr} - {slave}");
            match tokio::time::timeout(
                Duration::from_secs(SUNSPEC_DEVICE_CONNECT_TIMEOUT),
                SunSpecUnit::new(addr.clone(), slave, u.tls.clone()),
            )
            .await
            {
                Ok(good) => match good {
                    Ok(p) => {
                        info!("connected");
                        devices.push(p)
                    }
                    Err(e) => {
                        error!("Unable to create connection to SunSpec Unit: {e}");
                        retry_queue.push_back((addr, *s, Utc::now()));
                    }
                },
                Err(e) => {
                    error!("Timeout connecting to sunspec unit {addr}/{s}: {e}");
                    retry_queue.push_back((addr, *s, Utc::now()));
                }
            };
        }
    }
    //endregion

    //region create sunspec thread workers
    for d in devices {
        let mut tasks = TASK_PILE.write().await;
        let tx = tx.clone();
        let bcast_rx = broadcast_tx.clone().subscribe();
        let task_name = format!("poll_loop_{}", d.serial_number);
        let span = tracing::info_span!("task", name = task_name.as_str());
        let bar = task::Builder::new()
            .name(&format!("worker-{}", d.clone().serial_number))
            .spawn(
                async move {
                    match poll_loop(&d, tx, bcast_rx).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
                .instrument(span),
            );
        if bar.is_err() {
            error!("unit poll_loop crashed out");
        }
    }

    //endregion

    //region watch the mpsc tasks receive loop

    let mut msg_queue: VecDeque<PublishMessage> = VecDeque::new();
    let mut incoming_control_queue: VecDeque<InboundMessage> = VecDeque::new();
    loop {
        //endregion
        //region sunspec device channel loop handling
        while rx.len() > 0 {
            match rx.try_recv() {
                Ok(ipcm) => {
                    match ipcm {
                        IPCMessage::Shutdown => {
                            unreachable!();
                        }
                        IPCMessage::Outbound(o) => {
                            msg_queue.push_front(o);
                        }
                        IPCMessage::Error(e) => {
                            die(&format!("serial_number={}: {}", e.serial_number, e.msg));
                        }
                        IPCMessage::PleaseReconnect(addr, slave) => {
                            let tx = tx.clone();
                            let bcast_rx = broadcast_tx.subscribe();
                            let mut tls: Option<TlsConfig> = None;
                            warn!("Reconnect requested for {addr}/{slave}");
                            for u in config.units.clone() {
                                if u.addr == addr && u.slaves.contains(&slave) {
                                    tls = u.tls.clone();
                                    break;
                                }
                            }
                            let ssu: Option<SunSpecUnit> = match tokio::time::timeout(
                                Duration::from_secs(SUNSPEC_DEVICE_CONNECT_TIMEOUT),
                                SunSpecUnit::new(addr.clone(), slave.to_string(), tls),
                            )
                            .await
                            {
                                Ok(good) => match good {
                                    Ok(unit) => Some(unit),
                                    Err(e) => {
                                        warn!("{addr}:{slave} - Couldn't reconnect to sunspec unit: {e}");
                                        retry_queue.push_back((addr, slave, Utc::now()));
                                        None
                                    }
                                },
                                Err(e) => {
                                    warn!("{addr}:{slave} - Couldn't create new sunspecunit to replace dead conn: {e}");
                                    retry_queue.push_back((addr, slave, Utc::now()));
                                    None
                                }
                            };
                            if ssu.is_some() {
                                let unit = ssu.unwrap();
                                warn!(
                                    "{}/{} - Initial reconnection initiated, starting fresh task",
                                    unit.addr, unit.slave_id
                                );
                                let mut tasks = TASK_PILE.write().await;
                                let build = tokio::task::Builder::new();
                                let taskname = format!("worker-{}", unit.serial_number);
                                tasks
                                    .build_task()
                                    .name(&taskname.clone())
                                    .spawn(async move {
                                        match poll_loop(&unit, tx, bcast_rx).await {
                                            Ok(_) => {
                                                error!("Exited... OK? from the sunspec poll?  Unpossible!");
                                                Ok(())
                                            },
                                            Err(e) => {
                                                error!("{taskname} thread exited in error: {e}");
                                                return Err(e)
                                            },
                                        }
                                    })
                                    .unwrap();
                            } else {
                                error!("Reconnect was unsuccessful.");
                            }
                        }
                        IPCMessage::Inbound(_) => {
                            // we don't send inbounds to mqtt
                            unreachable!();
                        }
                    }
                }
                Err(_) => {}
            }
        }

        while from_mqtt_rx.len() > 0 {
            match from_mqtt_rx.try_recv() {
                Ok(recvd) => match recvd {
                    IPCMessage::Inbound(inmsg) => {
                        info!(
                            "Received payload for {},{},{}:{}",
                            inmsg.serial_number, inmsg.model, inmsg.point_name, inmsg.payload
                        );
                        incoming_control_queue.push_front(inmsg.clone());
                    }
                    IPCMessage::Shutdown => {
                        unreachable!();
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
            }
        }

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
                                Ok(_) => {
                                    error!("Task exited, which should not happen.  Hopefully reconnecting.")
                                }
                                Err(e) => {
                                    error!("Task exited with Ok(Err(e)): {e}");
                                }
                            },
                            Err(e) => {
                                // TODO: what does Err mean here?
                                error!("Got an error when checking joinset: {e}");
                            }
                        }
                    }
                    None => {
                        // No tasks waiting to report in
                    }
                }
            }
            None => {
                // no tasks are ready to be queried
            }
        }

        if mqtt_handler.is_finished() {
            die("MQTT thread exited.");
        }

        if tracer.is_some() {
            let t = tracer.clone().unwrap();
            let tp = t.provider().unwrap();
            let _rs = tp.force_flush();
        }

        //region retry connections
        // if there are pending reconnects, lets intentionally try just one of them per loop
        if let Some(conn) = retry_queue.pop_front() {
            let now = Utc::now();
            let then = conn.2;
            if now.signed_duration_since(then) > TimeDelta::seconds(30) {
                // we waited 30 seconds, lets try again
                if let Err(e) = tx.send(IPCMessage::PleaseReconnect(conn.0, conn.1)).await {
                    error!("Attempted to emit reconnect message, but failed: {e}");
                }
            } else {
                retry_queue.push_back(conn);
            }
        }
        //endregion
        let _ = sleep(Duration::from_millis(GENERIC_WAIT_MILLIS)).await;
    }
    //endregion
}

pub fn make_tracer(url: String, sample: f32) -> Tracer {
    let exporter = opentelemetry_otlp::new_exporter().http().with_endpoint(url);
    let batch_config = BatchConfig::default()
        .with_max_export_batch_size(1024)
        //.with_scheduled_delay(Duration::from_millis(5000))
        .with_max_export_timeout(Duration::from_secs(10))
        .with_max_queue_size(16384);
    let otlp_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        "sunspec_gateway",
                    ),
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                        env!("CARGO_PKG_VERSION"),
                    ),
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                        uuid::Uuid::new_v4().to_string(),
                    ),
                ]))
                .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(
                    sample as f64,
                )),
        )
        .with_batch_config(batch_config)
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Can't create tracer");
    otlp_tracer
}
