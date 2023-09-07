#[macro_use]
extern crate tracing;
extern crate strum;

mod cli_args;
mod date_serializer;
mod ipc;
mod monitored_point;
mod mqtt_connection;
mod mqtt_poll;
mod sunspec_poll;
mod sunspec_unit;

use crate::cli_args::CliArgs;
use crate::ipc::{IPCMessage, Payload, PublishMessage};
use crate::mqtt_connection::MqttConnection;
use crate::mqtt_poll::mqtt_poll_loop;
use crate::sunspec_poll::poll_loop;
use clap::Parser;
use config::Config;
use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use sunspec_unit::SunSpecUnit;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::sleep;
use tracing_log::AsTrace;
use tracing_subscriber;

const MPSC_BUFFER_SIZE: usize = 100_usize;

lazy_static! {
    //region create SETTINGS static object
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
            Ok(s) => s,
            Err(_e) => { "./config.toml".to_string()}
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
    let cli = CliArgs::parse();
    let (tx, mut rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (mqtt_tx, mut mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    // let waker = futures::task::noop_waker();
    // let mut cx = Context::from_waker(&waker);

    tracing_subscriber::fmt()
        .with_max_level(cli.verbose.log_level_filter().as_trace())
        .init();
    //endregion

    //region get config and load in pwrcell unit defs
    let config = SETTINGS.read().await;
    let units = match config.get_array("unit") {
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
    ) {
        Ok(m) => m,
        Err(e) => {
            return die("Couldn't create mqtt connection object: {e}");
        }
    };
    let mqtt_handler = tokio::task::spawn(async move {
        mqtt_poll_loop(mqtt_conn, mqtt_rx).await;
    });
    //endregion

    //region populate pwrcell devices into an array
    let mut devices: Vec<SunSpecUnit> = vec![];
    for u in units {
        let table = u.clone().into_table().unwrap();
        match SunSpecUnit::new(
            table
                .clone()
                .get("addr")
                .unwrap()
                .to_string()
                .parse()
                .unwrap(),
            table.clone().get("slave_id").unwrap().to_string(),
        )
        .await
        {
            Ok(p) => devices.push(p),
            Err(e) => {
                die(&format!("Unable to create connection to SunSpec Unit: {e}"));
            }
        };
    }
    //endregion

    //region create pwrcell thread workers
    let mut tasks = vec![];
    for d in devices {
        let tx = tx.clone();
        tasks.push(tokio::task::spawn(async move {
            poll_loop(&d, tx).await;
        }));
    }
    //endregion

    //region watch the mpsc tasks receive loop
    let mut msg_queue: VecDeque<PublishMessage> = VecDeque::new();
    loop {
        //endregion
        //region pwrcell device channel loop handling
        match rx.try_recv() {
            Ok(ipcm) => match ipcm {
                IPCMessage::Outbound(o) => {
                    msg_queue.push_front(o);
                }
                IPCMessage::Error(e) => {
                    die(&format!("serial_number={}: {}", e.serial_number, e.msg));
                }
            },
            Err(_) => {}
        }

        //endregion

        while let Some(msg) = msg_queue.pop_front() {
            if let Err(e) = mqtt_tx.send(IPCMessage::Outbound(msg)).await {
                error!("Unable to send mqtt message: {e}");
            }
        }
        let _ = sleep(Duration::from_millis(1000));
    }
    //endregion
}
