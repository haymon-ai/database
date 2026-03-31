//! MySQL/MariaDB backend crate.
//!
//! Provides [`MysqlBackend`] implementing the [`server::McpBackend`] trait.

mod connection;
mod operations;
mod schema;
mod tools;

pub use connection::MysqlBackend;
