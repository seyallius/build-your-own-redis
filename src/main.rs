//! Build Your Own X - Redis!

#![allow(unused_imports)]
use anyhow::{Context, Result};
use std::net::TcpListener;

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").context("Could not bind on 6379")?;
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
                continue;
            }
        }
    }
    Ok(())
}
