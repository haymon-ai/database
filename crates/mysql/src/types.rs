//! MySQL/MariaDB-specific MCP tool request types.
//!
//! The shared list request pairs (`listTables`, `listViews`, `listFunctions`,
//! `listProcedures`), the brief/detailed payload (`ListEntries`), and the general
//! `ListEntriesResponse` live in the shared `dbmcp-server` crate; they are re-exported
//! here so call sites can keep importing them from `crate::types`. Only the
//! MySQL-specific `dropTable` request remains defined here.

use schemars::JsonSchema;
use serde::Deserialize;

pub use dbmcp_server::types::{
    ListEntries, ListEntriesResponse, PinnedListFunctionsRequest, PinnedListProceduresRequest, PinnedListTablesRequest,
    PinnedListViewsRequest, UnpinnedListFunctionsRequest, UnpinnedListProceduresRequest, UnpinnedListTablesRequest,
    UnpinnedListViewsRequest,
};

/// Request for the `dropTable` tool.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(rename = "DropTableRequest")]
pub struct PinnedDropTableRequest {
    /// Name of the table to drop. Must be non-empty.
    pub table: String,
}

/// Request for the `dropTable` tool.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(rename = "DropTableRequest")]
pub struct UnpinnedDropTableRequest {
    #[serde(flatten)]
    pub inner: PinnedDropTableRequest,
    /// Database containing the table. Defaults to the active database.
    #[serde(default)]
    pub database: Option<String>,
}
