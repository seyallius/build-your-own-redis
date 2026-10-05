//! Build Your Own X - Redis!

use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use crate::resp::parser::Value;

mod resp;

// ------------------------------------------- <Main> ------------------------------------------- //

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
    const EOF: usize = 0;
    let mut buffer: Vec<u8> = Vec::new(); // accumulates across reads
    let mut chunk = [0u8; READ_BUF_SIZE]; // scratch space for one read

    loop {
        let bytes_read = stream
            .read(&mut chunk)
            .with_context(|| format!("Could not read from {peer}"))?;
        if bytes_read == EOF {
            break;
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        // Keep processing as long as there's a complete message in the buffer
        loop {
            let Some((value, read_bytes)) = resp::parse(&buffer)? else {
                break; // incomplete, wait for more data
            };

            buffer.drain(..read_bytes);
            // equivalent to:
            // buffer = buffer[read_bytes..].to_vec();
            match value {
                Value::Array(items) => {
                    if items.is_empty() {
                        stream.write_all(b"-ERR empty command\r\n")?;
                        continue;
                    }

                    let command = match &items[0] {
                        Value::Bulk(Some(bytes)) => bytes,
                        _ => {
                            stream.write_all(b"-ERR invalid command\r\n")?;
                            continue;
                        }
                    };

                    let command_lower = command.to_ascii_lowercase();
                    match command_lower.as_slice() {
                        b"ping" => {
                            stream
                                .write_all(b"+PONG\r\n")
                                .context("Could not send PING response")?;
                        }
                        b"echo" => {
                            if let Some(Value::Bulk(Some(arg_val))) = items.get(1) {
                                // bulk string format: $<length>\r\n<data>\r\n

                                let header = format!("${}\r\n", arg_val.len());
                                let mut response = header.into_bytes();
                                response.extend_from_slice(arg_val);
                                response.extend_from_slice(b"\r\n");

                                stream
                                    .write_all(&response)
                                    .context("Could not send ECHO response")?;
                            } else {
                                stream
                                    .write_all(
                                        b"-ERR wrong number of arguments for 'echo' command\r\n",
                                    )
                                    .context("Could not send error response")?;
                            }
                        }
                        b"command" => {
                            // redis-cli sends COMMAND DOCS on startup.
                            // Return an empty array to satisfy it.
                            stream
                                .write_all(b"*0\r\n")
                                .context("Could not empty array response")?
                        }
                        _ => {
                            let cmd_str = String::from_utf8_lossy(&command);
                            let err = format!("-ERR unknown command '{}'\r\n", cmd_str);
                            stream.write_all(err.as_bytes())?;
                        }
                    }
                }
                _ => {
                    // Inline commands (not array) — ignore or error
                    stream.write_all(b"-ERR expected array\r\n")?;
                }
            }
        }
    }
    Ok(())
}
