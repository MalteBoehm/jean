//! OpenCode CLI management module
//!
//! Handles detecting and checking the OpenCode CLI installation.
//! Unlike Claude CLI, we don't manage installation — OpenCode must be
//! pre-installed on the system.

mod commands;

pub use commands::*;
