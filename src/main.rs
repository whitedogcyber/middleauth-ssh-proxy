mod commands;
pub mod middleauth;
pub mod config;
pub mod token_storage;

use clap::{arg, Command};

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("proxy")
                .about("SSH Proxy Command")
                .arg(arg!(<HOST> "The host to connect to"))
                .arg(arg!(<PORT> "The port to connect to"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("configure")
                .about("SSH Proxy Configure")
                .subcommand_required(true)
                .subcommand(
                    Command::new("init")
                )
                .subcommand(
                    Command::new("auth")
                )

        )
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("proxy", args)) => commands::proxy::main(args),
        Some(("configure", args)) => commands::configure::main(args),
        _ => unreachable!()
    }
}
