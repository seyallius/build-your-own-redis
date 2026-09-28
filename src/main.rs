//! Build Your Own X - Redis!

use anyhow::{Context, Result};
use std::io::Read;
use std::{io::Write, net::TcpListener};

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").context("Could not bind on 6379")?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 512];
                loop {
                    let bytes_read = stream
                        .read(&mut buf)
                        .context(format!("Could not read from {}", stream.peer_addr()?))?;
                    if bytes_read == 0 {
                        break;
                    }
                    stream
                        .write_all(b"+PONG\r\n")
                        .context("Could not send PONG")?;
                }
            }
            Err(e) => {
                println!("error: {}", e);
                continue;
            }
        }
    }
    Ok(())
}
