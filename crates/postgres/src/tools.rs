//! MCP tool registration for the `PostgreSQL` backend.

use std::sync::Arc;

use backend::types::{CreateDatabaseRequest, GetTableSchemaRequest, ListTablesRequest, QueryRequest};
use rmcp::handler::server::common::{schema_for_empty_input, schema_for_type};
use rmcp::handler::server::router::tool::{ToolRoute, ToolRouter};
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, Tool, ToolAnnotations};
use rmcp::schemars::JsonSchema;
use serde_json::Value;
use server::server::map_error;
use server::{McpBackend, Server};
use tracing::info;

use rmcp::handler::server::common::FromContextPart;
use serde_json::Map as JsonObject;

use super::PostgresBackend;

/// Returns the JSON Schema for `Parameters<T>`.
fn schema_for<T: JsonSchema + 'static>() -> Arc<JsonObject<String, Value>> {
    schema_for_type::<Parameters<T>>()
}

impl PostgresBackend {
    /// Registers the `list_databases` tool.
    fn register_list_databases(&self, router: &mut ToolRouter<Server>) {
        let b = self.clone();
        router.add_route(ToolRoute::new_dyn(
            Tool::new(
                "list_databases",
                "List all accessible databases on the connected database server. Call this first to discover available database names.",
                schema_for_empty_input(),
            )
            .with_annotations(ToolAnnotations::new().read_only(true).destructive(false).idempotent(true).open_world(false)),
            move |_ctx: ToolCallContext<'_, Server>| {
                let b = b.clone();
                Box::pin(async move {
                    info!("TOOL: list_databases called");
                    let db_list = b.list_databases().await.map_err(map_error)?;
                    info!("TOOL: list_databases completed. Databases found: {}", db_list.len());
                    let json = serde_json::to_string_pretty(&db_list).unwrap_or_else(|_| "[]".into());
                    Ok(CallToolResult::success(vec![Content::text(json)]))
                })
            },
        ));
    }

    /// Registers the `list_tables` tool.
    fn register_list_tables(&self, router: &mut ToolRouter<Server>) {
        let b = self.clone();
        router.add_route(ToolRoute::new_dyn(
            Tool::new(
                "list_tables",
                "List all tables in a specific database. Requires database_name from list_databases.",
                schema_for::<ListTablesRequest>(),
            )
            .with_annotations(
                ToolAnnotations::new()
                    .read_only(true)
                    .destructive(false)
                    .idempotent(true)
                    .open_world(false),
            ),
            move |mut ctx: ToolCallContext<'_, Server>| {
                let params = Parameters::<ListTablesRequest>::from_context_part(&mut ctx);
                let b = b.clone();
                Box::pin(async move {
                    let params = params?;
                    let database_name = &params.0.database_name;
                    info!("TOOL: list_tables called. database_name={database_name}");
                    let table_list = b.list_tables(database_name).await.map_err(map_error)?;
                    info!("TOOL: list_tables completed. Tables found: {}", table_list.len());
                    let json = serde_json::to_string_pretty(&table_list).unwrap_or_else(|_| "[]".into());
                    Ok(CallToolResult::success(vec![Content::text(json)]))
                })
            },
        ));
    }

    /// Registers the `get_table_schema` tool.
    fn register_get_table_schema(&self, router: &mut ToolRouter<Server>) {
        let b = self.clone();
        router.add_route(ToolRoute::new_dyn(
            Tool::new(
                "get_table_schema",
                "Get column definitions (type, nullable, key, default) and foreign key relationships for a table. Requires database_name and table_name.",
                schema_for::<GetTableSchemaRequest>(),
            )
            .with_annotations(ToolAnnotations::new().read_only(true).destructive(false).idempotent(true).open_world(false)),
            move |mut ctx: ToolCallContext<'_, Server>| {
                let params = Parameters::<GetTableSchemaRequest>::from_context_part(&mut ctx);
                let b = b.clone();
                Box::pin(async move {
                    let params = params?;
                    let database_name = &params.0.database_name;
                    let table_name = &params.0.table_name;
                    info!("TOOL: get_table_schema called. database_name={database_name}, table_name={table_name}");
                    let schema = b.get_table_schema(database_name, table_name).await.map_err(map_error)?;
                    info!("TOOL: get_table_schema completed");
                    let json = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".into());
                    Ok(CallToolResult::success(vec![Content::text(json)]))
                })
            },
        ));
    }

    /// Registers the `read_query` tool.
    fn register_read_query(&self, router: &mut ToolRouter<Server>) {
        let b = self.clone();
        router.add_route(ToolRoute::new_dyn(
            Tool::new(
                "read_query",
                "Execute a read-only SQL query (SELECT, SHOW, DESCRIBE, USE, EXPLAIN).",
                schema_for::<QueryRequest>(),
            )
            .with_annotations(
                ToolAnnotations::new()
                    .read_only(true)
                    .destructive(false)
                    .idempotent(true)
                    .open_world(true),
            ),
            move |mut ctx: ToolCallContext<'_, Server>| {
                let params = Parameters::<QueryRequest>::from_context_part(&mut ctx);
                let b = b.clone();
                Box::pin(async move {
                    let params = params?;
                    let sql_query = &params.0.sql_query;
                    let database_name = &params.0.database_name;
                    info!(
                        "TOOL: execute_sql called. database_name={database_name}, sql_query={}",
                        &sql_query[..sql_query.len().min(100)]
                    );

                    {
                        let dialect = sqlparser::dialect::PostgreSqlDialect {};
                        backend::validation::validate_read_only_with_dialect(sql_query, &dialect).map_err(map_error)?;
                    }

                    let db = if database_name.is_empty() {
                        None
                    } else {
                        Some(database_name.as_str())
                    };
                    let results = b.execute_query(sql_query, db).await.map_err(map_error)?;
                    let row_count = results.as_array().map_or(0, Vec::len);
                    info!("TOOL: execute_sql completed. Rows returned: {row_count}");
                    let json = serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into());
                    Ok(CallToolResult::success(vec![Content::text(json)]))
                })
            },
        ));
    }

    /// Registers the `write_query` tool.
    fn register_write_query(&self, router: &mut ToolRouter<Server>) {
        let b = self.clone();
        router.add_route(ToolRoute::new_dyn(
            Tool::new(
                "write_query",
                "Execute a write SQL query (INSERT, UPDATE, DELETE, CREATE, ALTER, DROP).",
                schema_for::<QueryRequest>(),
            )
            .with_annotations(
                ToolAnnotations::new()
                    .read_only(false)
                    .destructive(true)
                    .idempotent(false)
                    .open_world(true),
            ),
            move |mut ctx: ToolCallContext<'_, Server>| {
                let params = Parameters::<QueryRequest>::from_context_part(&mut ctx);
                let b = b.clone();
                Box::pin(async move {
                    let params = params?;
                    let sql_query = &params.0.sql_query;
                    let database_name = &params.0.database_name;
                    info!(
                        "TOOL: execute_sql called. database_name={database_name}, sql_query={}",
                        &sql_query[..sql_query.len().min(100)]
                    );

                    let db = if database_name.is_empty() {
                        None
                    } else {
                        Some(database_name.as_str())
                    };
                    let results = b.execute_query(sql_query, db).await.map_err(map_error)?;
                    let row_count = results.as_array().map_or(0, Vec::len);
                    info!("TOOL: execute_sql completed. Rows returned: {row_count}");
                    let json = serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".into());
                    Ok(CallToolResult::success(vec![Content::text(json)]))
                })
            },
        ));
    }

    /// Registers the `create_database` tool.
    fn register_create_database(&self, router: &mut ToolRouter<Server>) {
        let b = self.clone();
        router.add_route(ToolRoute::new_dyn(
            Tool::new(
                "create_database",
                "Create a new database. Not supported for SQLite.",
                schema_for::<CreateDatabaseRequest>(),
            )
            .with_annotations(
                ToolAnnotations::new()
                    .read_only(false)
                    .destructive(false)
                    .idempotent(false)
                    .open_world(false),
            ),
            move |mut ctx: ToolCallContext<'_, Server>| {
                let params = Parameters::<CreateDatabaseRequest>::from_context_part(&mut ctx);
                let b = b.clone();
                Box::pin(async move {
                    let params = params?;
                    let database_name = &params.0.database_name;
                    info!("TOOL: create_database called for database: '{database_name}'");
                    let result = b.create_database(database_name).await.map_err(map_error)?;
                    info!("TOOL: create_database completed");
                    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into());
                    Ok(CallToolResult::success(vec![Content::text(json)]))
                })
            },
        ));
    }
}

impl McpBackend for PostgresBackend {
    fn register_tools(&self, router: &mut ToolRouter<Server>) {
        self.register_list_databases(router);
        self.register_list_tables(router);
        self.register_get_table_schema(router);
        self.register_read_query(router);

        if !self.read_only {
            self.register_write_query(router);
            self.register_create_database(router);
        }
    }
}
