//! Database backend trait for multi-database support.
//!
//! Defines the [`DatabaseBackend`] trait that all database implementations
//! (`MySQL`, `PostgreSQL`, `SQLite`) must satisfy.

use crate::error::AppError;
use async_trait::async_trait;
use serde_json::{Map, Value};
use sqlparser::dialect::Dialect;

/// Abstraction over database-specific operations.
///
/// Each supported database (`MySQL`, `PostgreSQL`, `SQLite`) implements this trait
/// to provide uniform access to schema exploration and query execution.
#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    /// Lists all accessible databases.
    async fn list_databases(&self) -> Result<Vec<String>, AppError>;

    /// Lists all tables in a database.
    async fn list_tables(&self, database: &str) -> Result<Vec<String>, AppError>;

    /// Returns column definitions for a table.
    async fn get_table_schema(&self, database: &str, table: &str) -> Result<Value, AppError>;

    /// Returns column definitions with foreign key relationships.
    async fn get_table_schema_with_relations(
        &self,
        database: &str,
        table: &str,
    ) -> Result<Value, AppError>;

    /// Executes a SQL query and returns rows as JSON objects.
    async fn execute_query(
        &self,
        sql: &str,
        database: Option<&str>,
    ) -> Result<Vec<Map<String, Value>>, AppError>;

    /// Creates a database if it doesn't exist.
    async fn create_database(&self, name: &str) -> Result<Value, AppError>;

    /// Returns the SQL dialect for this backend.
    fn dialect(&self) -> Box<dyn Dialect>;

    /// Whether read-only mode is enabled.
    fn read_only(&self) -> bool;
}
