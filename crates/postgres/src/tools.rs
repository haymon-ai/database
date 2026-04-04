//! MCP tool definitions for the `PostgreSQL` backend.
//!
//! Each tool is a unit struct implementing [`ToolBase`] and [`AsyncTool`].

use std::borrow::Cow;

use database_mcp_server::tools;
use database_mcp_server::types::{CreateDatabaseRequest, GetTableSchemaRequest, ListTablesRequest, QueryRequest};
use rmcp::handler::server::router::tool::{AsyncTool, ToolBase};
use rmcp::model::{ErrorData, ToolAnnotations};

use database_mcp_server::Server;

use super::PostgresAdapter;

/// Type alias kept module-private for brevity in tool impls.
type PostgresHandler = Server<PostgresAdapter>;

/// Tool to list all accessible databases.
pub(super) struct ListDatabasesTool;

impl ListDatabasesTool {
    const NAME: &'static str = "list_databases";
    const DESCRIPTION: &'static str = "List all accessible databases on the connected database server. Call this first to discover available database names.";
}

impl ToolBase for ListDatabasesTool {
    type Parameter = ();
    type Output = String;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn input_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
    }

    fn output_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
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

impl AsyncTool<PostgresHandler> for ListDatabasesTool {
    async fn invoke(handler: &PostgresHandler, _req: ()) -> Result<String, ErrorData> {
        tools::list_databases(handler.backend.list_databases()).await
    }
}

/// Tool to list all tables in a database.
pub(super) struct ListTablesTool;

impl ListTablesTool {
    const NAME: &'static str = "list_tables";
    const DESCRIPTION: &'static str =
        "List all tables in a specific database. Requires database_name from list_databases.";
}

impl ToolBase for ListTablesTool {
    type Parameter = ListTablesRequest;
    type Output = String;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn output_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
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

impl AsyncTool<PostgresHandler> for ListTablesTool {
    async fn invoke(handler: &PostgresHandler, req: ListTablesRequest) -> Result<String, ErrorData> {
        tools::list_tables(handler.backend.list_tables(&req.database_name), &req.database_name).await
    }
}

/// Tool to get column definitions for a table.
pub(super) struct GetTableSchemaTool;

impl GetTableSchemaTool {
    const NAME: &'static str = "get_table_schema";
    const DESCRIPTION: &'static str = "Get column definitions (type, nullable, key, default) and foreign key relationships for a table. Requires database_name and table_name.";
}

impl ToolBase for GetTableSchemaTool {
    type Parameter = GetTableSchemaRequest;
    type Output = String;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn output_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
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

impl AsyncTool<PostgresHandler> for GetTableSchemaTool {
    async fn invoke(handler: &PostgresHandler, req: GetTableSchemaRequest) -> Result<String, ErrorData> {
        tools::get_table_schema(
            handler.backend.get_table_schema(&req.database_name, &req.table_name),
            &req.database_name,
            &req.table_name,
        )
        .await
    }
}

/// Tool to execute a read-only SQL query.
pub(super) struct ReadQueryTool;

impl ReadQueryTool {
    const NAME: &'static str = "read_query";
    const DESCRIPTION: &'static str = "Execute a read-only SQL query (SELECT, SHOW, DESCRIBE, USE, EXPLAIN).";
}

impl ToolBase for ReadQueryTool {
    type Parameter = QueryRequest;
    type Output = String;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn output_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
    }

    fn annotations() -> Option<ToolAnnotations> {
        Some(
            ToolAnnotations::new()
                .read_only(true)
                .destructive(false)
                .idempotent(true)
                .open_world(true),
        )
    }
}

impl AsyncTool<PostgresHandler> for ReadQueryTool {
    async fn invoke(handler: &PostgresHandler, req: QueryRequest) -> Result<String, ErrorData> {
        tools::read_query(
            handler
                .backend
                .execute_query(&req.sql_query, tools::resolve_database(&req.database_name)),
            &req.sql_query,
            &req.database_name,
            |sql| {
                database_mcp_sql::validation::validate_read_only_with_dialect(
                    sql,
                    &sqlparser::dialect::PostgreSqlDialect {},
                )
            },
        )
        .await
    }
}

/// Tool to execute a write SQL query.
pub(super) struct WriteQueryTool;

impl WriteQueryTool {
    const NAME: &'static str = "write_query";
    const DESCRIPTION: &'static str = "Execute a write SQL query (INSERT, UPDATE, DELETE, CREATE, ALTER, DROP).";
}

impl ToolBase for WriteQueryTool {
    type Parameter = QueryRequest;
    type Output = String;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn output_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
    }

    fn annotations() -> Option<ToolAnnotations> {
        Some(
            ToolAnnotations::new()
                .read_only(false)
                .destructive(true)
                .idempotent(false)
                .open_world(true),
        )
    }
}

impl AsyncTool<PostgresHandler> for WriteQueryTool {
    async fn invoke(handler: &PostgresHandler, req: QueryRequest) -> Result<String, ErrorData> {
        tools::write_query(
            handler
                .backend
                .execute_query(&req.sql_query, tools::resolve_database(&req.database_name)),
            &req.sql_query,
            &req.database_name,
        )
        .await
    }
}

/// Tool to create a new database.
pub(super) struct CreateDatabaseTool;

impl CreateDatabaseTool {
    const NAME: &'static str = "create_database";
    const DESCRIPTION: &'static str = "Create a new database. Not supported for SQLite.";
}

impl ToolBase for CreateDatabaseTool {
    type Parameter = CreateDatabaseRequest;
    type Output = String;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some(Self::DESCRIPTION.into())
    }

    fn output_schema() -> Option<std::sync::Arc<rmcp::model::JsonObject>> {
        None
    }

    fn annotations() -> Option<ToolAnnotations> {
        Some(
            ToolAnnotations::new()
                .read_only(false)
                .destructive(false)
                .idempotent(false)
                .open_world(false),
        )
    }
}

impl AsyncTool<PostgresHandler> for CreateDatabaseTool {
    async fn invoke(handler: &PostgresHandler, req: CreateDatabaseRequest) -> Result<String, ErrorData> {
        tools::create_database(handler.backend.create_database(&req.database_name), &req.database_name).await
    }
}
