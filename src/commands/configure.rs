use clap::ArgMatches;
use crate::config::{get_config, write_config};

pub fn main(args: &ArgMatches) {
    match args.subcommand() {
        Some(("init", args)) => init(args),
        Some(("auth", args)) => auth(args),
        _ => unreachable!()
    }
}

fn init(args: &ArgMatches) {
    let config = get_config();
    write_config(&config).expect("Failed to write config");
}

fn auth(args: &ArgMatches) {
    todo!()
}