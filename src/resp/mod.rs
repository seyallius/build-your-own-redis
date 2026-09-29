//! # Redis Serialization Protocol (RESP) — Message Format
//!
//! This module documents how a raw RESP message is structured on the wire.
//! The example below shows a complete request consisting of two bulk strings
//! inside a single array. The message is written using explicit escape
//! sequences so that every byte is visible.
//!
//! ```text
//! *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
//! ```
//!
//! ## Byte-by-byte breakdown
//!
//! | Bytes  | Meaning                                                        |
//! |--------|----------------------------------------------------------------|
//! | `*2`   | "An array of 2 things is coming."                              |
//! | `\r\n` | End of the `2` line.                                           |
//! | `$4`   | "The next element is a bulk string of 4 bytes."                |
//! | `\r\n` | End of the `4` line.                                           |
//! | `ECHO` | Exactly 4 bytes — the actual string.                           |
//! | `\r\n` | End of that string.                                            |
//! | `$3`   | "The next element is a bulk string of 3 bytes."                |
//! | `\r\n` | End of that line.                                              |
//! | `hey`  | Exactly 3 bytes.                                               |
//! | `\r\n` | End of that string.                                            |
//!
//! ## Result
//!
//! Parsing the message above should yield:
//!
//! ```text
//! ["ECHO", "hey"]
//! ```
//!
//! ## Protocol symbols
//!
//! RESP uses a single leading byte to tell the parser what kind of value is
//! coming next. The first byte of every value is therefore significant.
//!
//! | Symbol | Meaning                                                                 |
//! |--------|-------------------------------------------------------------------------|
//! | `*`    | An array is coming. Followed by a count.                                |
//! | `$`    | A bulk string is coming. Followed by its length in bytes.               |
//! | `+`    | A simple string (like `+PONG`). Followed by the string itself.          |
//! | `:`    | An integer (like `:42`).                                                |
//! | `-`    | An error message.                                                       |
//!
//! ## Notes
//!
//! - Every line in RESP is terminated by the two-byte sequence `\r\n`
//!   (carriage return followed by line feed). This terminator is never
//!   counted as part of a bulk string's length.
//! - Bulk strings are binary-safe: the length in bytes is authoritative, so
//!   the payload may contain `\r\n` or any other byte without ambiguity.
//! - Array counts and bulk string lengths are always decimal integers encoded
//!   as ASCII text, not as raw binary numbers.
//! - Simple strings and errors cannot contain `\r` or `\n`, which is why they
//!   do not carry an explicit length.

pub(crate) mod parser;

pub(crate) use parser::parse;
