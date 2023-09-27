use clap::Parser;
use clap_verbosity_flag;

#[derive(Parser)]
#[command(author = "Peter Grace <pete.grace@gmail.com>")]
#[command(about = "Monitor SunSpec-compliant devices over modbus-TCP")]
pub struct CliArgs {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
    #[arg(short = 'c', long = "config_path")]
    pub config_path: Option<String>,
    #[arg(short = 'd', long = "db_path")]
    pub db_path: Option<String>,
}
