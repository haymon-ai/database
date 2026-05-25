//! PostgreSQL-specific MCP tool request types.
//!
//! The shared list request pair (`PinnedListEntriesRequest` / `UnpinnedListEntriesRequest`,
//! used by `listTables`, `listViews`, `listTriggers`, `listFunctions`, `listProcedures`,
//! `listMaterializedViews`), the brief/detailed payload (`ListEntries`), and the general
//! `ListEntriesResponse` live in the `dbmcp-server` crate; they are re-exported here so
//! call sites can keep importing them from `crate::types`. Only the Postgres-specific
//! `dropTable` (with `cascade`) request remains defined here.

use schemars::JsonSchema;
use serde::Deserialize;

pub use dbmcp_server::types::{ListEntries, ListEntriesResponse, PinnedListEntriesRequest, UnpinnedListEntriesRequest};

/// Request for the `dropTable` tool.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(rename = "DropTableRequest")]
pub struct PinnedDropTableRequest {
    /// Name of the table to drop. Must be non-empty.
    pub table: String,
    /// If true, use CASCADE to also drop dependent foreign key constraints. Defaults to false.
    #[serde(default)]
    pub cascade: bool,
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
