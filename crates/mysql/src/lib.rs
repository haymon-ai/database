//! MySQL/MariaDB backend crate.
//!
//! Provides [`MysqlBackend`] for database operations with MCP
//! tool registration via [`Backend`](database_mcp_server::Backend).

mod connection;
mod operations;
mod schema;
mod server;
mod tools;

pub use connection::MysqlBackend;
