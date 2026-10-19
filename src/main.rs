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
    env,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    thread,
};

mod commands;
mod resp;
mod server;
mod store;

/// Bind address for the Redis-compatible listener.
const DEFAULT_PORT: u16 = 6379;

/// Process entry point.
///
/// Parses the optional port argument, binds the listener to localhost, and
/// spawns a thread for each accepted client connection. Accept errors are
/// logged so the server can continue accepting connections.
fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let port = parse_port()?;
    let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

    let listener =
        TcpListener::bind(address).with_context(|| format!("could not bind on {address}"))?;

    let store = Store::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = store.clone();
                thread::spawn(move || {
                    if let Err(e) = server::connection::handle(stream, &store) {
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
