//! Database backend enum for multi-database support.
//!
//! Defines the [`Backend`] enum that dispatches to concrete
//! `MySQL`, `PostgreSQL`, or `SQLite` implementations without dynamic dispatch.

use crate::db::mysql::MysqlBackend;
use crate::db::postgres::PostgresBackend;
use crate::db::sqlite::SqliteBackend;
use crate::error::AppError;
use serde_json::{Map, Value};
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};

/// Concrete database backend — no dynamic dispatch.
#[derive(Clone)]
pub enum Backend {
    /// `MySQL`/`MariaDB` via sqlx.
    Mysql(MysqlBackend),
    /// `PostgreSQL` via sqlx.
    Postgres(PostgresBackend),
    /// `SQLite` via sqlx.
    Sqlite(SqliteBackend),
}

impl Backend {
    /// Lists all accessible databases.
    pub async fn list_databases(&self) -> Result<Vec<String>, AppError> {
        match self {
            Self::Mysql(b) => b.list_databases().await,
            Self::Postgres(b) => b.list_databases().await,
            Self::Sqlite(b) => b.list_databases().await,
        }
    }

    /// Lists all tables in a database.
    pub async fn list_tables(&self, database: &str) -> Result<Vec<String>, AppError> {
        match self {
            Self::Mysql(b) => b.list_tables(database).await,
            Self::Postgres(b) => b.list_tables(database).await,
            Self::Sqlite(b) => b.list_tables(database).await,
        }
    }

    /// Returns column definitions for a table.
    pub async fn get_table_schema(&self, database: &str, table: &str) -> Result<Value, AppError> {
        match self {
            Self::Mysql(b) => b.get_table_schema(database, table).await,
            Self::Postgres(b) => b.get_table_schema(database, table).await,
            Self::Sqlite(b) => b.get_table_schema(database, table).await,
        }
    }

    /// Returns column definitions with foreign key relationships.
    pub async fn get_table_schema_with_relations(
        &self,
        database: &str,
        table: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Mysql(b) => b.get_table_schema_with_relations(database, table).await,
            Self::Postgres(b) => b.get_table_schema_with_relations(database, table).await,
            Self::Sqlite(b) => b.get_table_schema_with_relations(database, table).await,
        }
    }

    /// Executes a SQL query and returns rows as JSON objects.
    pub async fn execute_query(
        &self,
        sql: &str,
        database: Option<&str>,
    ) -> Result<Vec<Map<String, Value>>, AppError> {
        match self {
            Self::Mysql(b) => b.execute_query(sql, database).await,
            Self::Postgres(b) => b.execute_query(sql, database).await,
            Self::Sqlite(b) => b.execute_query(sql, database).await,
        }
    }

    /// Creates a database if it doesn't exist.
    pub async fn create_database(&self, name: &str) -> Result<Value, AppError> {
        match self {
            Self::Mysql(b) => b.create_database(name).await,
            Self::Postgres(b) => b.create_database(name).await,
            Self::Sqlite(b) => b.create_database(name).await,
        }
    }

    /// Returns the SQL dialect for this backend.
    #[must_use]
    pub fn dialect(&self) -> Box<dyn Dialect> {
        match self {
            Self::Mysql(_) => Box::new(MySqlDialect {}),
            Self::Postgres(_) => Box::new(PostgreSqlDialect {}),
            Self::Sqlite(_) => Box::new(SQLiteDialect {}),
        }
    }

    /// Whether read-only mode is enabled.
    #[must_use]
    pub fn read_only(&self) -> bool {
        match self {
            Self::Mysql(b) => b.read_only,
            Self::Postgres(b) => b.read_only,
            Self::Sqlite(b) => b.read_only,
        }
    }
}
