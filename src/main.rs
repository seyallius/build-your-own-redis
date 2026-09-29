//! Build Your Own X - Redis!

use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").context("Could not bind on 6379")?;
    for stream in listener.incoming() {
        let stream = stream.context("accepting connection failed")?;
        thread::spawn(move || {
            if let Err(e) = handle_client(stream) {
                eprintln!("error handling client: {e:#}");
            }
        });
    }
    Ok(())
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Serves one client: logs the peer, then replies `+PONG\r\n` for every
/// non-empty read until the client disconnects.
fn handle_client(mut stream: TcpStream) -> Result<()> {
    //TODO(event-loop): "To implement this, you'll need to either use threads or, if you're feeling adventurous,
    // implement an Event Loop (like the official Redis implementation does)." - After finishing the
    // challenge, get back and re-implement with event loop (tokio).

    let peer = stream.peer_addr().context("peer_addr failed")?;
    println!("connection accepted for: {}", peer.ip().to_canonical());

    const READ_BUF_SIZE: usize = 512;
    let mut buf = [0; READ_BUF_SIZE];
    const EOF: usize = 0;
    loop {
        let bytes_read = stream
            .read(&mut buf)
            .with_context(|| format!("Could not read from {peer}"))?;
        if bytes_read == EOF {
            break;
        }

        stream
            .write_all(b"+PONG\r\n")
            .context("Could not send PONG")?;
    }
    Ok(())
}
