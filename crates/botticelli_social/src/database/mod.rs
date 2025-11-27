//! Database command executor for narrative-driven database operations.
//!
//! This module provides bot commands for database operations like updating tables,
//! enabling narratives to manage application state through structured commands.

mod commands;

pub use commands::DatabaseCommandExecutor;
