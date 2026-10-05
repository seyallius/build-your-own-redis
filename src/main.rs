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

use anyhow::{Context, Result};
use std::{net::TcpListener, thread};

mod commands;
mod resp;
mod server;

/// Bind address for the Redis-compatible listener.
///
/// Kept as a constant so it can be overridden in one place (e.g. for tests or
/// when adding CLI argument parsing later).
const BIND_ADDRESS: &str = "127.0.0.1:6379";

/// Process entry point.
///
/// Binds [`BIND_ADDRESS`], accepts connections forever, and spawns one OS
/// thread per client. Errors during `accept` are logged and the loop
/// continues so a single bad connection cannot bring the server down.
fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind(BIND_ADDRESS)
        .with_context(|| format!("could not bind on {BIND_ADDRESS}"))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(e) = server::connection::handle(stream) {
                        eprintln!("error handling client: {e:#}");
                    }
                });
            }
            Err(e) => eprintln!("accept failed: {e:#}"),
        }
    }

    Ok(())
}
