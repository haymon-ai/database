//! MCP tool: `listTriggers`.

use std::borrow::Cow;

use dbmcp_server::pagination::Pager;
use dbmcp_sql::Connection;
use rmcp::handler::server::router::tool::{AsyncTool, ToolBase};
use rmcp::model::{ErrorData, ToolAnnotations};

use crate::PostgresHandler;
use crate::types::{ListTriggersRequest, ListTriggersResponse, TriggerEntries};

/// Marker type for the `listTriggers` MCP tool.
pub(crate) struct ListTriggersTool;

impl ListTriggersTool {
    const NAME: &'static str = "listTriggers";
    const TITLE: &'static str = "List Triggers";
    const DESCRIPTION: &'static str = r#"List user-defined triggers in the `public` schema, optionally filtered and/or with full metadata.

<usecase>
Use when:
- Auditing triggers across a database (brief mode, default).
- Searching for a trigger by partial name (pass `search`).
- Inspecting a trigger's timing, events, activation level, handler function, status, and full `CREATE TRIGGER` text before reasoning about side-effects (pass `detailed: true`). Detailed mode supersedes ad-hoc `readQuery` against `pg_trigger` / `information_schema.triggers`.
</usecase>

<parameters>
- `database` — Database to target. Defaults to the active database.
- `cursor` — Opaque pagination cursor; echo the prior response's `nextCursor`.
- `search` — Case-insensitive filter on trigger names via `ILIKE`. `%` matches any sequence; `_` matches a single character.
- `detailed` — When `true`, returns full metadata objects keyed by trigger name instead of bare name strings. Default `false`.
</parameters>

<examples>
✓ "What triggers are in the mydb database?" → listTriggers(database="mydb")
✓ "Find the audit triggers" → listTriggers(search="audit")
✓ "What does orders_audit_after_iu do?" → listTriggers(search="orders_audit_after_iu", detailed=true)
✗ "Show me a trigger's body" → use detailed mode; the `definition` field carries the full `CREATE TRIGGER` text
</examples>

<what_it_returns>
Brief mode (default): a sorted JSON array of trigger-name strings, e.g. `["customers_audit_after_insert", "orders_audit_after_insert"]`.
Detailed mode: a JSON object keyed by trigger name; each value carries `schema`, `table`, `status` (ENABLED/DISABLED/REPLICA/ALWAYS), `timing` (BEFORE/AFTER/INSTEAD OF), `events` (array of strings drawn from INSERT/UPDATE/DELETE/TRUNCATE in that fixed order), `activationLevel` (ROW/STATEMENT), `functionName`, and `definition` (the full `CREATE TRIGGER` text). Internal triggers (FK enforcement etc.) are excluded.
</what_it_returns>

<pagination>
Paginated. Pass the prior response's `nextCursor` as `cursor` to fetch the next page. The `search` filter must stay the same across pages for cursor continuity.
</pagination>"#;
}

impl ToolBase for ListTriggersTool {
    type Parameter = ListTriggersRequest;
    type Output = ListTriggersResponse;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn title() -> Option<String> {
        Some(Self::TITLE.into())
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn annotations() -> Option<ToolAnnotations> {
        Some(
            ToolAnnotations::new()
                .read_only(true)
                .destructive(false)
                .idempotent(true)
                .open_world(false),
        )
    }
}

impl AsyncTool<PostgresHandler> for ListTriggersTool {
    async fn invoke(handler: &PostgresHandler, params: Self::Parameter) -> Result<Self::Output, Self::Error> {
        handler.list_triggers(params).await
    }
}

/// Brief-mode SQL: `pg_trigger` join with optional `ILIKE` filter on trigger name.
const BRIEF_SQL: &str = r"
    SELECT t.tgname
    FROM pg_trigger t
    JOIN pg_class c ON t.tgrelid = c.oid
    JOIN pg_namespace n ON c.relnamespace = n.oid
    WHERE n.nspname = 'public'
      AND NOT t.tgisinternal
      AND ($1::text IS NULL OR t.tgname ILIKE '%' || $1 || '%')
    ORDER BY t.tgname
    LIMIT $2 OFFSET $3";

/// Detailed-mode SQL: single CTE returning one `json_build_object` per trigger.
///
/// `LIMIT`/`OFFSET` are pushed into `trigger_page` so downstream evaluation
/// (function lookup, definition formatting, events array) only runs for
/// the at-most `page_size + 1` triggers actually on this page.
const DETAILED_SQL: &str = r"
    WITH trigger_page AS (
        SELECT
            t.oid          AS trigger_oid,
            n.nspname      AS schema_name,
            c.relname      AS table_name,
            t.tgname       AS trigger_name,
            t.tgenabled    AS enabled_code,
            t.tgtype::int  AS type_bits,
            t.tgfoid       AS function_oid
        FROM pg_trigger t
        JOIN pg_class c     ON t.tgrelid = c.oid
        JOIN pg_namespace n ON c.relnamespace = n.oid
        WHERE NOT t.tgisinternal
          AND n.nspname = 'public'
          AND ($1::text IS NULL OR t.tgname ILIKE '%' || $1 || '%')
        ORDER BY t.tgname, c.relname, n.nspname
        LIMIT $2 OFFSET $3
    )
    SELECT
        tp.trigger_name AS name,
        json_build_object(
            'schema',          tp.schema_name,
            'table',           tp.table_name,
            'status',          CASE tp.enabled_code
                                   WHEN 'O' THEN 'ENABLED'
                                   WHEN 'D' THEN 'DISABLED'
                                   WHEN 'R' THEN 'REPLICA'
                                   WHEN 'A' THEN 'ALWAYS'
                               END,
            'timing',          CASE
                                   WHEN (tp.type_bits & 2)  = 2  THEN 'BEFORE'
                                   WHEN (tp.type_bits & 64) = 64 THEN 'INSTEAD OF'
                                   ELSE 'AFTER'
                               END,
            'events',          (
                SELECT COALESCE(json_agg(e.ev ORDER BY e.ord), '[]'::json)
                FROM (VALUES
                    ('INSERT'::text,   1, 4),
                    ('UPDATE'::text,   2, 16),
                    ('DELETE'::text,   3, 8),
                    ('TRUNCATE'::text, 4, 32)
                ) AS e(ev, ord, mask)
                WHERE (tp.type_bits & e.mask) = e.mask
            ),
            'activationLevel', CASE WHEN (tp.type_bits & 1) = 1 THEN 'ROW' ELSE 'STATEMENT' END,
            'functionName',    p.proname,
            'definition',      pg_get_triggerdef(tp.trigger_oid)
        ) AS entry
    FROM trigger_page tp
    LEFT JOIN pg_proc p ON p.oid = tp.function_oid
    ORDER BY tp.trigger_name, tp.table_name, tp.schema_name";

impl PostgresHandler {
    /// Lists one page of user-defined triggers, optionally filtered and/or detailed.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorData`] with code `-32602` if `cursor` is malformed,
    /// or an internal-error [`ErrorData`] if `database` is invalid
    /// or the underlying query fails.
    pub async fn list_triggers(
        &self,
        ListTriggersRequest {
            database,
            cursor,
            search,
            detailed,
        }: ListTriggersRequest,
    ) -> Result<ListTriggersResponse, ErrorData> {
        let database = database.as_deref().map(str::trim).filter(|s| !s.is_empty());

        let pager = Pager::new(cursor, self.config.page_size);
        let pattern = search.as_deref().map(str::trim).filter(|s| !s.is_empty());

        if detailed {
            return self.list_triggers_detailed(database, pattern, pager).await;
        }

        self.list_triggers_brief(database, pattern, pager).await
    }

    /// Brief-mode page: sorted trigger-name strings wrapped as [`TriggerEntries::Brief`].
    async fn list_triggers_brief(
        &self,
        database: Option<&str>,
        pattern: Option<&str>,
        pager: Pager,
    ) -> Result<ListTriggersResponse, ErrorData> {
        let rows: Vec<String> = self
            .connection
            .fetch_scalar(
                sqlx::query(BRIEF_SQL)
                    .bind(pattern)
                    .bind(pager.limit())
                    .bind(pager.offset()),
                database,
            )
            .await?;
        let (triggers, next_cursor) = pager.finalize(rows);
        Ok(ListTriggersResponse {
            triggers: TriggerEntries::Brief(triggers),
            next_cursor,
        })
    }

    /// Detailed-mode page: name-keyed metadata wrapped as [`TriggerEntries::Detailed`].
    async fn list_triggers_detailed(
        &self,
        database: Option<&str>,
        pattern: Option<&str>,
        pager: Pager,
    ) -> Result<ListTriggersResponse, ErrorData> {
        let rows: Vec<(String, sqlx::types::Json<serde_json::Value>)> = self
            .connection
            .fetch(
                sqlx::query(DETAILED_SQL)
                    .bind(pattern)
                    .bind(pager.limit())
                    .bind(pager.offset()),
                database,
            )
            .await?;
        let (rows, next_cursor) = pager.finalize(rows);
        Ok(ListTriggersResponse {
            triggers: TriggerEntries::Detailed(rows.into_iter().map(|(name, json)| (name, json.0)).collect()),
            next_cursor,
        })
    }
}
