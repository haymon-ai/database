//! `SQLite` connection configuration and backend definition.

use backend::error::AppError;
use config::DatabaseConfig;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tracing::info;

/// `SQLite` file-based database backend.
#[derive(Clone)]
pub struct SqliteBackend {
    pub(crate) pool: SqlitePool,
    pub read_only: bool,
}

impl std::fmt::Debug for SqliteBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteBackend")
            .field("read_only", &self.read_only)
            .finish_non_exhaustive()
    }
}

impl SqliteBackend {
    /// Creates a lazy in-memory backend for tests.
    #[cfg(test)]
    pub(crate) fn in_memory(read_only: bool) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_lazy("sqlite::memory:")
            .expect("in-memory SQLite");
        Self { pool, read_only }
    }

    /// Creates a new `SQLite` backend from configuration.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Connection`] if the database file cannot be opened.
    pub async fn new(config: &DatabaseConfig) -> Result<Self, AppError> {
        let name = config.name.as_deref().unwrap_or_default();
        let pool = SqlitePoolOptions::new()
            .max_connections(1) // SQLite is single-writer
            .connect_with(connect_options(config))
            .await
            .map_err(|e| AppError::Connection(format!("Failed to open SQLite: {e}")))?;

        info!("SQLite connection initialized: {name}");

        Ok(Self {
            pool,
            read_only: config.read_only,
        })
    }

    /// Wraps `name` in double quotes for safe use in `SQLite` SQL statements.
    ///
    /// Escapes internal double quotes by doubling them.
    pub(crate) fn quote_identifier(name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

/// Builds [`SqliteConnectOptions`] from a [`DatabaseConfig`].
fn connect_options(config: &DatabaseConfig) -> SqliteConnectOptions {
    let name = config.name.as_deref().unwrap_or_default();
    SqliteConnectOptions::new().filename(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::DatabaseBackend;

    #[test]
    fn quote_identifier_wraps_in_double_quotes() {
        assert_eq!(SqliteBackend::quote_identifier("users"), "\"users\"");
        assert_eq!(SqliteBackend::quote_identifier("eu-docker"), "\"eu-docker\"");
    }

    #[test]
    fn quote_identifier_escapes_double_quotes() {
        assert_eq!(SqliteBackend::quote_identifier("test\"db"), "\"test\"\"db\"");
        assert_eq!(SqliteBackend::quote_identifier("a\"b\"c"), "\"a\"\"b\"\"c\"");
    }

    #[test]
    fn try_from_sets_filename() {
        let config = DatabaseConfig {
            backend: DatabaseBackend::Sqlite,
            name: Some("test.db".into()),
            ..DatabaseConfig::default()
        };
        let opts = connect_options(&config);

        assert_eq!(opts.get_filename().to_str().expect("valid path"), "test.db");
    }

    #[test]
    fn try_from_empty_name_defaults() {
        let config = DatabaseConfig {
            backend: DatabaseBackend::Sqlite,
            name: None,
            ..DatabaseConfig::default()
        };
        let opts = connect_options(&config);

        // Empty string filename — validated elsewhere by Config::validate()
        assert_eq!(opts.get_filename().to_str().expect("valid path"), "");
    }
}
