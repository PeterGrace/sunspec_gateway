#[macro_use] extern crate tracing;
extern crate strum;

mod pwrcell_unit;
mod cli_args;
mod monitored_point;

use pwrcell_unit::PWRCellUnit;
use crate::cli_args::CliArgs;
use clap::Parser;
use tracing_log::AsTrace;
use tracing_subscriber;
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use config::Config;
use std::process;
use futures::future::join_all;
use crate::pwrcell_unit::poll_loop;

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
            Ok(s) => s,
            Err(_e) => { "./pwrcell.toml".to_string()}
        };
        let settings = match Config::builder()
            .add_source(config::File::with_name(&cfg_file))
            .add_source(
                config::Environment::with_prefix("PWRCELL")
                .try_parsing(true)
                .list_separator(",")
                .with_list_parse_key("supported_architectures")
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
}


#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();
    // setup log level
    tracing_subscriber::fmt()
        .with_max_level(cli.verbose.log_level_filter().as_trace())
        .init();
    let config = SETTINGS.read().await;
    let units = match config.get_array("unit") {
        Ok(u) => u,
        Err(e) => {
            error!("unable to get unit definitions from config file: {e}");
            process::exit(1);
        }
    };
    let mut devices: Vec<PWRCellUnit> = vec![];
    for u in units {
        let table = u.clone().into_table().unwrap();
        match PWRCellUnit::new(
            table.clone().get("addr").unwrap().to_string().parse().unwrap(),
            table.clone().get("slave_id").unwrap().to_string(),
        ).await {
            Ok(p) => devices.push(p),
            Err(e) => {
                error!("Unable to create PWRCellUnit: {e}");
                process::exit(1);
            }
        };
    }
    let mut tasks = vec![];
    for d in devices {
        tasks.push(tokio::task::spawn(async move {
            poll_loop(&d).await;
        }));
    }
    let result = futures::future::join_all(tasks).await;
}
