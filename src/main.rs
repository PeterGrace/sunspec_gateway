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
use console_subscriber;
use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use sunspec_unit::SunSpecUnit;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing_log::AsTrace;
use tracing_subscriber::prelude::*;

const MPSC_BUFFER_SIZE: usize = 100_usize;

lazy_static! {
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
    let cli = CliArgs::parse();
    let (tx, mut rx) = mpsc::channel(MPSC_BUFFER_SIZE);
    let (mqtt_tx, mut mqtt_rx) = mpsc::channel(MPSC_BUFFER_SIZE);

    if Some(true) == cli.ttrace {
        let console_layer = console_subscriber::spawn();
        tracing_subscriber::registry()
            .with(console_layer)
            .with(tracing_subscriber::fmt::layer().with_level(true))
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(cli.verbose.log_level_filter().as_trace())
            .init();
    }

    //endregion

    //region get config and load in sunspec unit defs
    let config = SETTINGS.read().await;
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
    let mut tasks = vec![];
    for d in devices {
        let tx = tx.clone();
        tasks.push(tokio::task::spawn(async move {
            match poll_loop(&d, tx).await {
                Ok(_) => info!("poll_loop exited ok."),
                Err(e) => die(&format!("poll died: {e}")),
            };
        }));
    }
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

        let _ = sleep(Duration::from_millis(1000));
    }
    //endregion
}
