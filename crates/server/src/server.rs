//! Shared MCP server utilities.
//!
//! Provides [`server_info`] used by per-backend
//! [`ServerHandler`](rmcp::ServerHandler) implementations and the
//! binary crate's `ServerHandler` wrapper.

use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};

/// Hardcoded product name matching the root binary crate.
const NAME: &str = "database-mcp";

/// The current version, derived from the workspace `Cargo.toml` at compile time.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-readable title for the MCP server.
const TITLE: &str = "Database MCP Server";

/// Product description surfaced to MCP clients.
const DESCRIPTION: &str = "Database MCP Server for MySQL, MariaDB, PostgreSQL, and SQLite";

/// Website URL, derived from the workspace `Cargo.toml` at compile time.
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// Returns the shared [`ServerInfo`] for all server implementations.
///
/// Builds a complete [`Implementation`] with product metadata.
#[must_use]
pub fn server_info() -> ServerInfo {
    let capabilities = ServerCapabilities::builder().enable_tools().build();

    ServerInfo::new(capabilities).with_server_info(
        Implementation::new(NAME, VERSION)
            .with_title(TITLE)
            .with_description(DESCRIPTION)
            .with_website_url(HOMEPAGE),
    )
}
