//! SQLite-specific MCP tool request types.
//!
//! Unlike `MySQL` and `PostgreSQL`, `SQLite` operates on a single file and has no
//! database selection. The shared `Pinned*` request types carry no `database` field,
//! so `SQLite` reuses them directly (re-exported under the unprefixed names) for
//! `listTables`, `listTriggers`, `writeQuery`, and `readQuery`. The brief/detailed
//! payload (`ListEntries`) and the general `ListEntriesResponse` are likewise shared.
//! The `listViews` (cursor-only), `explainQuery` (no `analyze`), and `dropTable`
//! (no `cascade`) requests have SQLite-specific shapes and remain defined here.

use dbmcp_server::pagination::Cursor;
use schemars::JsonSchema;
use serde::Deserialize;

pub use dbmcp_server::types::{
    ListEntries, ListEntriesResponse, PinnedListTablesRequest as ListTablesRequest,
    PinnedListTriggersRequest as ListTriggersRequest, PinnedQueryRequest as QueryRequest,
    PinnedReadQueryRequest as ReadQueryRequest,
};

/// Request for the `dropTable` tool.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct DropTableRequest {
    /// Name of the table to drop. Must be non-empty.
    pub table: String,
}

/// Request for the `listViews` tool.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListViewsRequest {
    /// Opaque pagination cursor. Omit (or pass `null`) for the first page.
    /// On subsequent calls, pass the `nextCursor` returned by the previous
    /// response verbatim. Cursors are opaque — do not parse, modify, or persist.
    #[serde(default)]
    pub cursor: Option<Cursor>,
}

/// Request for the `explainQuery` tool.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ExplainQueryRequest {
    /// The SQL query to explain.
    pub query: String,
}

#[cfg(test)]
mod tests {
    use super::{ListTablesRequest, ListTriggersRequest};

    #[test]
    fn list_tables_request_defaults_to_brief_mode_without_search() {
        let req: ListTablesRequest = serde_json::from_str("{}").expect("empty object should parse");
        assert!(req.search.is_none());
        assert!(!req.detailed, "detailed must default to false");
    }

    #[test]
    fn list_tables_request_accepts_search_and_detailed() {
        let req: ListTablesRequest = serde_json::from_str(r#"{"search": "post", "detailed": true}"#).expect("parse");
        assert_eq!(req.search.as_deref(), Some("post"));
        assert!(req.detailed);
    }

    #[test]
    fn list_triggers_request_defaults_to_brief_mode_without_search() {
        let req: ListTriggersRequest = serde_json::from_str("{}").expect("empty object should parse");
        assert!(req.search.is_none());
        assert!(!req.detailed, "detailed must default to false");
    }

    #[test]
    fn list_triggers_request_accepts_search_and_detailed() {
        let req: ListTriggersRequest = serde_json::from_str(r#"{"search": "audit", "detailed": true}"#).expect("parse");
        assert_eq!(req.search.as_deref(), Some("audit"));
        assert!(req.detailed);
    }
}
