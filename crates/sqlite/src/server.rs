//! MCP handler for the `SQLite` backend.
//!
//! Implements [`Backend`] on [`SqliteAdapter`] to register
//! SQLite-specific MCP tools.

use database_mcp_server::{Backend, Server};
use rmcp::handler::server::tool::ToolRouter;

use super::SqliteAdapter;
use super::tools::{GetTableSchemaTool, ListTablesTool, ReadQueryTool, WriteQueryTool};

impl Backend for SqliteAdapter {
    fn provide_tool_router(&self) -> ToolRouter<Server<Self>> {
        let router = ToolRouter::new()
            .with_async_tool::<ListTablesTool>()
            .with_async_tool::<GetTableSchemaTool>()
            .with_async_tool::<ReadQueryTool>();

        if self.config.read_only {
            return router;
        }

        router.with_async_tool::<WriteQueryTool>()
    }
}
