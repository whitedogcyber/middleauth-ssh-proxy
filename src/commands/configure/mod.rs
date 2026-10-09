pub mod init;
pub mod auth;

use clap::ArgMatches;
use crate::commands::configure::auth::auth;
use crate::commands::configure::init::init;

pub fn main(args: &ArgMatches) {
    match args.subcommand() {
        Some(("init", args)) => init(args),
        Some(("auth", args)) => auth(args),
        _ => unreachable!()
    }
}