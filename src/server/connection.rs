//! Per-client TCP read loop.
//!
//! The loop is deliberately small: read bytes, append them to a buffer, and
//! hand complete RESP frames to [`dispatch::dispatch`]. Any partially-received
//! frame stays in the buffer until more bytes arrive.

use crate::{
    resp::{self, parser::Value},
    server::dispatch,
};
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

/// Size of the scratch buffer used for each `read(2)` call.
///
/// This only bounds the size of a single syscall; the accumulation buffer
/// ([`Vec<u8>`]) grows as needed to hold a full frame.
const READ_BUF_SIZE: usize = 512;

/// Return value of [`std::io::Read::read`] signalling end-of-stream.
const EOF: usize = 0;

// ------------------------------------- Public (crate) API ------------------------------------- //

/// Serves a single client until it disconnects or a fatal I/O error occurs.
///
/// The function owns the [`TcpStream`] for its lifetime and returns `Ok(())`
/// on a clean disconnect. Parser-level errors are reported to the client as
/// RESP errors and terminate the connection; I/O errors are propagated so the
/// caller (a thread in `main`) can log them.
pub(crate) fn handle(mut stream: TcpStream) -> Result<()> {
    let peer = stream.peer_addr().context("peer_addr failed")?;
    println!("connection accepted for: {}", peer.ip().to_canonical());

    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; READ_BUF_SIZE];

    loop {
        let bytes_read = stream
            .read(&mut chunk)
            .with_context(|| format!("could not read from {peer}"))?;
        if bytes_read == EOF {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        // Drain every complete frame currently in the buffer.
        while let Some((value, consumed)) = next_frame(&mut stream, &buffer, peer)? {
            buffer.drain(..consumed);
            dispatch::dispatch(&mut stream, value)?;
        }
    }

    Ok(())
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Attempts to parse one complete RESP frame from `buffer`.
///
/// * Returns `Ok(Some((value, consumed)))` when a frame is ready.
/// * Returns `Ok(None)` when `buffer` holds only a partial frame.
/// * Returns `Err` on a genuine protocol error, after writing a RESP error
///   reply to the client so it does not hang waiting for a response.
fn next_frame(
    stream: &mut TcpStream,
    buffer: &[u8],
    peer: std::net::SocketAddr,
) -> Result<Option<(Value, usize)>> {
    match resp::parse(buffer) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            eprintln!("parse error from {peer}: {e:#}");
            // Best-effort reply; ignore secondary write errors.
            let _ = stream.write_all(b"-ERR protocol error\r\n");
            Err(e)
        }
    }
}
