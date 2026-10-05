//! Command dispatcher.
//!
//! Maps a parsed RESP [`Value::Array`] onto a registered [`Command`] and runs
//! it. The dispatcher itself is command-agnostic: it consults
//! [`commands::lookup`] for the handler, so adding a new command is a
//! one-line change in the [`crate::commands`] registry.

use crate::{commands, resp::parser::Value};
use anyhow::{Context, Result};
use std::{io::Write, net::TcpStream};

// ------------------------------------- Public (crate) API ------------------------------------- //

/// Executes the command encoded by `value` and writes its reply to `stream`.
///
/// Expects `value` to be a non-empty RESP array whose first element is a bulk
/// string command name. Anything else is reported to the client as an error;
/// the connection is not closed.
pub(crate) fn dispatch(stream: &mut TcpStream, value: Value) -> Result<()> {
    let items = match value {
        Value::Array(items) => items,
        _ => return reply_error(stream, "expected array"),
    };

    if items.is_empty() {
        return reply_error(stream, "empty command");
    }

    let name = match &items[0] {
        Value::Bulk(Some(bytes)) => bytes.clone(),
        _ => return reply_error(stream, "invalid command"),
    };

    let Some(cmd) = commands::lookup(&name) else {
        let lossy = String::from_utf8_lossy(&name);
        return reply_error(stream, &format!("unknown command '{lossy}'"));
    };

    let args = &items[1..];
    let reply = cmd
        .execute(args)
        .with_context(|| format!("command '{}' failed", cmd.name()))?;

    write_value(stream, &reply)
}

/// Serializes a [`Value`] as a RESP reply and writes it to `stream`.
///
/// All command handlers produce a [`Value`]; this is the single place where a
/// [`Value`] becomes bytes on the wire.
pub(crate) fn write_value(stream: &mut TcpStream, value: &Value) -> Result<()> {
    let bytes = encode(value);
    stream.write_all(&bytes).context("could not write reply")
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Convenience helper for emitting a RESP `-ERR` reply.
fn reply_error(stream: &mut TcpStream, msg: &str) -> Result<()> {
    let line = format!("-ERR {msg}\r\n");
    stream
        .write_all(line.as_bytes())
        .context("could not write error reply")
}

/// Encodes a [`Value`] into its RESP wire representation.
///
/// This mirrors the encoding of the `Display` impl but produces bytes and
/// always terminates with `\r\n`, so it can be written straight to a socket.
fn encode(value: &Value) -> Vec<u8> {
    match value {
        Value::Simple(s) => format!("+{s}\r\n").into_bytes(),
        Value::Error(s) => format!("-{s}\r\n").into_bytes(),
        Value::Integer(n) => format!(":{n}\r\n").into_bytes(),
        Value::Bulk(None) => b"$-1\r\n".to_vec(),
        Value::Bulk(Some(bytes)) => {
            let mut out = format!("${}\r\n", bytes.len()).into_bytes();
            out.extend_from_slice(bytes);
            out.extend_from_slice(b"\r\n");
            out
        }
        Value::Array(items) => {
            let mut out = format!("*{}\r\n", items.len()).into_bytes();
            for item in items {
                out.extend_from_slice(&encode(item));
            }
            out
        }
    }
}
