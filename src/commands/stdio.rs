//! Stdio transport command.
//!
//! Runs the MCP server over stdin/stdout for use with Claude Desktop,
//! Cursor, and other MCP clients that communicate via stdio.

use clap::Parser;
use dbmcp_config::{Config, ConfigError, DatabaseConfig, PiiConfig};
use rmcp::ServiceExt;
use tracing::{error, info};

use crate::commands::common::{self, DatabaseArguments, PiiArguments};
use crate::error::Error;

/// Runs the MCP server in stdio mode.
#[derive(Debug, Parser)]
pub(crate) struct StdioCommand {
    /// Shared database connection flags.
    #[command(flatten)]
    db_arguments: DatabaseArguments,

    /// Shared PII flags.
    #[command(flatten)]
    pii_arguments: PiiArguments,
}

impl TryFrom<&StdioCommand> for Config {
    type Error = Vec<ConfigError>;

    fn try_from(cmd: &StdioCommand) -> Result<Self, Self::Error> {
        let mut errors: Vec<ConfigError> = Vec::new();
        let database = match DatabaseConfig::try_from(&cmd.db_arguments) {
            Ok(c) => Some(c),
            Err(e) => {
                errors.extend(e);
                None
            }
        };
        let pii = PiiConfig::from(&cmd.pii_arguments);
        if let Err(e) = pii.validate() {
            errors.extend(e);
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            database: database.expect("database config present when no errors collected"),
            http: None,
            pii,
        })
    }
}

impl StdioCommand {
    /// Builds the database configuration, server, and runs the stdio transport.
    ///
    /// Serves JSON-RPC over stdin/stdout.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails, the stdio
    /// transport fails to initialize, or the server encounters a fatal
    /// protocol error.
    pub(crate) async fn execute(&self) -> Result<(), Error> {
        let config = Config::try_from(self)?;
        let server = common::create_server(&config);

        info!("Starting MCP server via stdio transport...");
        let transport = rmcp::transport::io::stdio();
        let running = server.serve(transport).await?;
        if let Err(join_error) = running.waiting().await {
            error!("stdio server task terminated abnormally: {join_error}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dbmcp_config::DatabaseBackend;

    #[track_caller]
    fn parse(args: &[&str]) -> StdioCommand {
        StdioCommand::try_parse_from(args).expect("valid stdio command")
    }

    #[test]
    fn db_read_only_defaults_to_true() {
        let cmd = parse(&["_"]);
        assert!(cmd.db_arguments.read_only);
    }

    #[test]
    fn db_query_timeout_zero_passes_through() {
        let cmd = parse(&["_", "--db-query-timeout", "0"]);
        let config = DatabaseConfig::try_from(&cmd.db_arguments).expect("valid db args");
        assert_eq!(config.query_timeout, Some(0));
    }

    #[test]
    fn db_args_populate_database_config() {
        let cmd = parse(&["_", "--db-backend", "postgres", "--db-user", "pg", "--db-name", "app"]);
        assert_eq!(cmd.db_arguments.backend, DatabaseBackend::Postgres);
        assert_eq!(cmd.db_arguments.user.as_deref(), Some("pg"));
        assert_eq!(cmd.db_arguments.name.as_deref(), Some("app"));

        let config = DatabaseConfig::try_from(&cmd.db_arguments).expect("valid postgres args");
        assert_eq!(config.backend, DatabaseBackend::Postgres);
        assert_eq!(config.user, "pg");
        assert_eq!(config.name.as_deref(), Some("app"));
    }

    #[test]
    fn try_from_database_arguments_propagates_validation_errors() {
        // SQLite without --db-name must fail validation inside the TryFrom impl,
        // surfacing `ConfigError::MissingSqliteDbName` to the caller.
        let cmd = parse(&["_", "--db-backend", "sqlite"]);
        let errors =
            DatabaseConfig::try_from(&cmd.db_arguments).expect_err("sqlite without --db-name must be rejected");
        assert!(errors.iter().any(|e| matches!(e, ConfigError::MissingSqliteDbName)));
    }
}
