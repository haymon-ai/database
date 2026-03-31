//! `SQLite` backend crate.
//!
//! Provides [`SqliteBackend`] implementing the [`server::McpBackend`] trait.

mod connection;
mod operations;
mod schema;
mod tools;

pub use connection::SqliteBackend;
