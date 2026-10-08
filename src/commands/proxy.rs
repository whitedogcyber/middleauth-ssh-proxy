use std::net::TcpStream;
use std::{io, thread};
use std::io::{Read, Write};
use clap::ArgMatches;
use crate::middleauth::cert::ensure_ssh_certificate;

pub fn main(args: &ArgMatches) {
    let host = args.get_one::<String>("HOST").expect("A HOST is required");
    let port = args.get_one::<u16>("PORT").expect("A PORT is required");

    if let Err(e) = ensure_ssh_certificate(host.as_str()) {
        eprintln!("Failed to ensure valid SSH certificate: {}", e);
        std::process::exit(1);
    }

    start_pipe(host.as_str(), port)
}

fn start_pipe(host: &str, port: &u16) {
    let address = format!("{}:{}", host, port);

    let mut socket_reader = match TcpStream::connect(&address) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", address, e);
            std::process::exit(1);
        }
    };

    let mut socket_writer = socket_reader.try_clone().expect("Failed to clone TCP stream");

    // Thread A: Pipe Remote Server (Socket) -> Local Machine (Stdout)
    let handle = thread::spawn(move || {
        let mut buffer = [0; 8192];
        let mut stdout = io::stdout();
        while let Ok(n) = socket_reader.read(&mut buffer) {
            if n == 0 { break; } // Connection closed by remote
            if stdout.write_all(&buffer[..n]).is_err() { break; }
            let _ = stdout.flush();
        }
    });

    // Thread B (Main Thread): Pipe Local Machine (Stdin) -> Remote Server (Socket)
    let mut buffer = [0; 8192];
    let mut stdin = io::stdin();
    while let Ok(n) = stdin.read(&mut buffer) {
        if n == 0 { break; } // Connection closed by local SSH client
        if socket_writer.write_all(&buffer[..n]).is_err() { break; }
        let _ = socket_writer.flush();
    }

    // Wait for the background thread to finish cleanly
    let _ = handle.join();
}