use clap::Parser;
use clap_verbosity_flag;

#[derive(Parser)]
#[command(author = "Peter Grace <pete.grace@gmail.com>")]
#[command(about = "Monitor PWRCell devices over modbus-TCP")]
pub struct CliArgs {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}
