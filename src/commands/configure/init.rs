use clap::ArgMatches;
use crate::config::{get_config, write_config};

pub fn init(_args: &ArgMatches) {
    let config = get_config();
    write_config(&config).expect("Failed to write config");
}