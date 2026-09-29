//! A parser for the [RESP (REdis Serialization Protocol)][resp] wire format.
//!
//! RESP is the protocol used by Redis clients and servers to exchange data. Every
//! message begins with a single **type byte** that identifies the kind of value being
//! sent, followed by type-specific payload bytes and a terminating `\r\n`.
//!
//! This module provides an incremental, non-allocating-where-possible parser that
//! consumes a byte slice and, when a complete value is available, returns it together
//! with the number of bytes that were consumed. This makes it suitable for use with
//! buffered I/O, where partial messages may arrive across multiple reads.
//!
//! The supported RESP types are:
//!
//! | Byte | Name        | Description                                 |
//! |------|-------------|---------------------------------------------|
//! | `+`  | Simple      | A non-binary, non-null UTF-8 string.        |
//! | `-`  | Error       | A non-binary, non-null UTF-8 error string.  |
//! | `:`  | Integer     | A signed 64-bit integer.                    |
//! | `$`  | Bulk        | A binary-safe string, or `nil`.             |
//! | `*`  | Array       | A (possibly nested) sequence of values.     |
//!
//! [resp]: https://redis.io/docs/reference/protocol-spec/

use anyhow::Result;
use std::str;

// ----------------------------------- Types, Variables & Constants ----------------------------- //

/// RESP type marker for a **Simple String** (`+`).
///
/// Simple strings are used for short, non-binary safe replies such as `OK` or
/// `PONG`. The payload is UTF-8 and terminated by `\r\n`.
const SIMPLE: char = '+';

/// RESP type marker for an **Error** (`-`).
///
/// Error replies share the same shape as simple strings but are intended to
/// signal failure to the client (e.g. `-ERR unknown command`).
const ERROR: char = '-';

/// RESP type marker for an **Integer** (`:`).
///
/// Integers are encoded as a signed decimal number followed by `\r\n`.
const INTEGER: char = ':';

/// RESP type marker for a **Bulk String** (`$`).
///
/// Bulk strings are binary-safe: their length is transmitted explicitly so the
/// payload may contain arbitrary bytes, including `\r` and `\n`. A length of
/// `-1` denotes a `nil` bulk string.
const BULK: char = '$';

/// RESP type marker for an **Array** (`*`).
///
/// Arrays are introduced by their element count. A count of `-1` denotes a
/// `nil` array, while `0` denotes an empty array.
const ARRAY: char = '*';

/// A decoded RESP value.
///
/// Each variant corresponds to one of the RESP type bytes and carries the
/// already-parsed payload. Binary-safe types (`Bulk`, `Array`) use `Vec<u8>`
/// rather than `String` because Redis values are arbitrary byte sequences and
/// are not guaranteed to be valid UTF-8.
pub(crate) enum Value {
    /// The `+` value.
    ///
    /// A non-binary-safe, UTF-8 encoded string that cannot contain `\r` or `\n`.
    Simple(String),
    /// The `-` value.
    ///
    /// A non-binary-safe, UTF-8 encoded error message, encoded identically to
    /// [`Value::Simple`] but semantically indicating a failure.
    Error(String),
    /// The `:` value.
    ///
    /// A signed 64-bit integer parsed from its decimal ASCII representation.
    Integer(i64),
    /// The `$` value. Bulk strings are binary, not UTF-8. Redis keys and values are arbitrary bytes,
    /// that is why the usage of `Vec<u8>`.
    ///
    /// The `None` variant represents a *null* (nil) bulk string, which RESP
    /// encodes with a length of `-1`.
    Bulk(Option<Vec<u8>>),
    /// The `*` value.
    ///
    /// A sequence of zero or more [`Value`]s. Arrays may be arbitrarily nested.
    Array(Vec<Value>),
}

// ------------------------------------- Public (crate) API ------------------------------------- //

pub(crate) fn parse(input: &[u8]) -> Result<Option<(Value, usize)>> {
    let Some(&first) = input.first() else {
        anyhow::bail!("empty input received");
    };
    let marker = first as char;
    match marker {
        BULK => parse_bulk(input),
        SIMPLE => parse_simple(input),
        INTEGER => parse_integer(input),
        ERROR => parse_error(input),
        ARRAY => parse_array(input),
        _ => anyhow::bail!("unknown type byte: {:?}", input[0]),
    }
}

// -------------------------------------- Internal Helpers -------------------------------------- //

fn parse_bulk(_input: &[u8]) -> Result<Option<(Value, usize)>> {
    unimplemented!("implement me!")
}

fn parse_simple(_input: &[u8]) -> Result<Option<(Value, usize)>> {
    unimplemented!("implement me!")
}

fn parse_integer(_input: &[u8]) -> Result<Option<(Value, usize)>> {
    unimplemented!("implement me!")
}

fn parse_error(_input: &[u8]) -> Result<Option<(Value, usize)>> {
    unimplemented!("implement me!")
}

fn parse_array(input: &[u8]) -> Result<Option<(Value, usize)>> {
    // 1. Find the end of the array header:            *2\r\n...
    if let Some(r_index) = find_crlf_sequence(input)//    ^--- This is index 2
        && r_index > 1
    {
        let count_text = str::from_utf8(&input[1..r_index])?;
        let item_count: i64 = count_text.parse()?; // parsed `2`

        if item_count < 0 {
            anyhow::bail!("nil arrays not supported");
        }
        let item_count = item_count as usize;

        //TODO(empty array header): reject empty array header `*0\r\n`

        // 2. Start just after that header. *2\r\n\$4...
        let mut position = r_index + 2; //         ^--- This is index 4

        // 3. Parse one value, then another, using parse for each.
        let mut parsed_items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            if let Some((value, byte_count)) = parse(&input[position..])? {
                parsed_items.push(value);
                position += byte_count
            } else {
                return Ok(None); // the input is incomplete, so return Ok(None) immediately
            };
        }

        // 4. Return the two values and the total number of bytes consumed.
        return Ok(Some((Value::Array(parsed_items), position)));
    };
    Ok(None)
}

fn find_crlf_sequence(input: &[u8]) -> Option<usize> {
    if input.len() < 2 {
        return None;
    }
    for index in 0..input.len() - 1 {
        if input[index] == b'\r' && input[index + 1] == b'\n' {
            return Some(index);
        }
    }
    None
    // or: input.windows(2).position(|w| w == b"\r\n")
}

// ------------------------------------------ <Tests> ------------------------------------------- //

#[cfg(test)]
mod parser_tests {
    use super::*;

    // "<what's set up> → <what should happen>""

    mod parse_array {
        use super::*;

        #[test]
        fn proper_name_suggestion() {
            let input = b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
            let (value, consumed) = parse_array(input).unwrap().unwrap();
        }
    }

    mod find_crlf_sequence {
        use super::*;

        #[test]
        fn empty_input_has_no_crlf() {
            assert_eq!(find_crlf_sequence(&[]), None);
        }

        #[test]
        fn single_byte_input_has_no_crlf() {
            assert_eq!(find_crlf_sequence(&[b'\r']), None);
        }

        #[test]
        fn finds_crlf_not_at_start() {
            let input = &[b'*', b'\r', b'\n'];
            assert_eq!(find_crlf_sequence(input).unwrap(), 1);
        }

        #[test]
        fn returns_none_when_no_crlf_present() {
            let input = &[b'a', b'b'];
            assert_eq!(find_crlf_sequence(input), None);
        }

        #[test]
        fn trailing_cr_without_lf_does_not_panic() {
            assert_eq!(find_crlf_sequence(&[b'x', b'\r']), None);
        }
    }
}
