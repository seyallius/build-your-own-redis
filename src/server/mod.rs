//! Connection handling and command dispatch.
//!
//! This module groups the two responsibilities that used to live inside
//! `main.rs`:
//!
//! * [`connection`] — reading bytes off a [`std::net::TcpStream`] and feeding
//!   them into the incremental RESP parser.
//! * [`dispatch`]   — turning a parsed RESP array into a concrete command
//!   handler and executing it.
//!
//! Splitting them means the read loop never needs to know which commands
//! exist, and the dispatcher never needs to know about sockets.

pub(crate) mod connection;
pub(crate) mod dispatch;
