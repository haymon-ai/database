//! PostgreSQL-specific MCP tool request types.
//!
//! The shared list request pairs (`listTables`, `listViews`, `listFunctions`,
//! `listProcedures`, `listTriggers`), the brief/detailed payload (`ListEntries`), and the
//! general `ListEntriesResponse` live in the `dbmcp-server` crate; they are re-exported
//! here so call sites can keep importing them from `crate::types`. Only the
//! Postgres-specific `dropTable` (with `cascade`) and `listMaterializedViews` requests
//! remain defined here.

use dbmcp_server::pagination::Cursor;
use schemars::JsonSchema;
use serde::Deserialize;

pub use dbmcp_server::types::{
    ListEntries, ListEntriesResponse, PinnedListFunctionsRequest, PinnedListProceduresRequest, PinnedListTablesRequest,
    PinnedListTriggersRequest, PinnedListViewsRequest, UnpinnedListFunctionsRequest, UnpinnedListProceduresRequest,
    UnpinnedListTablesRequest, UnpinnedListTriggersRequest, UnpinnedListViewsRequest,
};

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

/// Request for the Postgres `listMaterializedViews` tool — extends the shared shape with `search` and `detailed`.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(rename = "ListMaterializedViewsRequest")]
pub struct PinnedListMaterializedViewsRequest {
    /// Opaque cursor from a prior response's `nextCursor`; omit for the first page.
    #[serde(default)]
    pub cursor: Option<Cursor>,
    /// Optional case-insensitive filter on matview names. The input is used within an `ILIKE`
    /// clause: `%` matches any sequence of characters and `_` matches any single character.
    #[serde(default)]
    pub search: Option<String>,
    /// When `true`, each returned entry is a full metadata object (schema, owner,
    /// description, definition, populated, indexed); when `false` or omitted, each
    /// entry is the bare matview-name string.
    #[serde(default)]
    pub detailed: bool,
}

/// Request for the Postgres `listMaterializedViews` tool — extends the shared shape with `search` and `detailed`.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(rename = "ListMaterializedViewsRequest")]
pub struct UnpinnedListMaterializedViewsRequest {
    #[serde(flatten)]
    pub inner: PinnedListMaterializedViewsRequest,
    /// Database to list materialized views from. Defaults to the active database.
    #[serde(default)]
    pub database: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{PinnedListMaterializedViewsRequest, UnpinnedListMaterializedViewsRequest};

    #[test]
    fn unpinned_list_materialized_views_request_defaults_to_brief_mode_without_search() {
        let req: PinnedListMaterializedViewsRequest = serde_json::from_str("{}").expect("empty object should parse");
        assert!(req.search.is_none());
        assert!(!req.detailed, "detailed must default to false");
    }

    #[test]
    fn unpinned_list_materialized_views_request_accepts_search_and_detailed() {
        let req: PinnedListMaterializedViewsRequest =
            serde_json::from_str(r#"{"search": "orders", "detailed": true}"#).expect("parse");
        assert_eq!(req.search.as_deref(), Some("orders"));
        assert!(req.detailed);
    }

    #[test]
    fn pinned_list_materialized_views_request_accepts_database() {
        let req: UnpinnedListMaterializedViewsRequest = serde_json::from_str(r#"{"database": "mydb"}"#).expect("parse");
        assert_eq!(req.database.as_deref(), Some("mydb"));
    }
}
