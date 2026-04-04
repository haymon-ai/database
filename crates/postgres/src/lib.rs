//! `PostgreSQL` backend crate.
//!
//! Provides [`PostgresAdapter`] for database operations with MCP
//! tool registration via [`Backend`](database_mcp_server::Backend).

mod adapter;
mod operations;
mod schema;
mod server;
mod tools;

pub use adapter::PostgresAdapter;
