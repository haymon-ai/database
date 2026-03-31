//! `PostgreSQL` backend crate.
//!
//! Provides [`PostgresBackend`] implementing the [`server::McpBackend`] trait.

mod connection;
mod operations;
mod schema;
mod tools;

pub use connection::PostgresBackend;
