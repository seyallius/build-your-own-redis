//! Build Your Own X - Redis!
//!
//! Entry point for `built-your-own-redis`.
//!
//! This crate implements a minimal Redis-compatible server speaking the
//! [RESP][resp] wire protocol. The `main` function is intentionally thin: it
//! binds a TCP listener and delegates every accepted connection to
//! [`server::connection::handle`] on its own thread.
//!
//! The actual protocol parsing lives in [`resp`], and the per-command logic
//! lives in [`commands`]. Keeping `main` free of business logic makes both of
//! those modules independently testable and easier to evolve.
//!
//! [resp]: https://redis.io/docs/reference/protocol-spec/

use crate::store::Store;
use anyhow::{Context, Result};
use std::{
    collections::HashMap, env, net::{Ipv4Addr, SocketAddrV4, TcpListener}, sync::{Arc, Mutex}, thread,
};

mod commands;
mod resp;
mod server;
mod store;

/// Bind address for the Redis-compatible listener.
const DEFAULT_PORT: u16 = 6379;

/// Process entry point.
///
/// Binds [`BIND_ADDRESS`], accepts connections forever, and spawns one OS
/// thread per client. Errors during `accept` are logged and the loop
/// continues so a single bad connection cannot bring the server down.
fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let port = parse_port()?;
    let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

    let listener =
        TcpListener::bind(address).with_context(|| format!("could not bind on {address}"))?;

    let storage = Store::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let storage = Arc::clone(&storage);
                thread::spawn(move || {
                    if let Err(e) = server::connection::handle(stream, &storage) {
                        eprintln!("error handling client: {e:#}");
                    }
                });
            }
            Err(e) => eprintln!("accept failed: {e:#}"),
        }
    }

    Ok(())
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Parses the optional --port argument, defaulting to 6379.
fn parse_port() -> Result<u16> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None => Ok(DEFAULT_PORT),
        Some("--port" | "-p") => {
            let value = args.next().context("missing value for --port")?;
            anyhow::ensure!(args.next().is_none(), "unexpected additional arguments");

            let port = value.parse::<u16>().context("invalid port number")?;
            anyhow::ensure!(port != 0, "port must be greater than zero");
            Ok(port)
        }
        Some(argument) => anyhow::bail!("unknown argument: {argument}"),
    }
}
