//! Command registry.
//!
//! Every Redis command the server understands is represented by a type
//! implementing [`Command`]. Commands are registered in [`REGISTRY`] and
//! looked up by name via [`lookup`].
//!
//! Adding a new command requires two lines:
//!
//! 1. Create a new module under `commands/` defining the handler type.
//! 2. Add the type to [`REGISTRY`].
//!
//! Nothing in [`crate::server`] needs to change.

use crate::resp::parser::Value;
use anyhow::Result;

pub(crate) mod command;
pub(crate) mod echo;
pub(crate) mod ping;

/// Static registry of all supported commands.
///
/// Linear lookup is fine for the handful of commands this server implements;
/// swap for a `HashMap` if the registry ever grows large.
const REGISTRY: &[&dyn Command] = &[&ping::Ping, &echo::Echo, &command::CommandCmd];

// ------------------------------------- Public (crate) API ------------------------------------- //

/// A single Redis command handler.
///
/// Implementors receive the command's arguments (everything after the command
/// name in the RESP array) and produce a [`Value`] describing the reply.
/// Returning `Err` is reserved for genuine internal failures; user-facing
/// errors should be returned as [`Value::Error`].
pub(crate) trait Command: Send + Sync + 'static {
    /// Canonical, lowercase command name used for dispatch.
    fn name(&self) -> &'static str;

    /// Executes the command against the given arguments.
    fn execute(&self, args: &[Value]) -> Result<Value>;
}

/// Looks up a command by its wire name.
///
/// Matching is ASCII-case-insensitive to mirror Redis' behavior.
pub(crate) fn lookup(name: &[u8]) -> Option<&'static dyn Command> {
    REGISTRY
        .iter()
        .copied()
        .find(|cmd| cmd.name().as_bytes().eq_ignore_ascii_case(name))
}
