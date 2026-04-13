//! Stdio transport command.
//!
//! Runs the MCP server over stdin/stdout for use with Claude Desktop,
//! Cursor, and other MCP clients that communicate via stdio.

use clap::Parser;
use database_mcp_config::DatabaseConfig;
use rmcp::ServiceExt;
use tracing::{error, info};

use crate::commands::common::{self, DatabaseArguments};
use crate::error::Error;

/// Runs the MCP server in stdio mode.
#[derive(Debug, Parser)]
pub(crate) struct StdioCommand {
    /// Shared database connection flags.
    #[command(flatten)]
    pub(crate) db_arguments: DatabaseArguments,
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
        let db_config = DatabaseConfig::try_from(&self.db_arguments)?;
        let server = common::create_server(&db_config);

        info!("Starting MCP server via stdio transport...");
        let transport = rmcp::transport::io::stdio();
        let running = server.serve(transport).await?;
        if let Err(join_error) = running.waiting().await {
            error!("stdio server task terminated abnormally: {join_error}");
        }
        Ok(())
    }
}
