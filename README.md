# sql-mcp

A single-binary [MCP](https://modelcontextprotocol.io/) server for SQL databases. Connect your AI assistant to MySQL/MariaDB, PostgreSQL, or SQLite with zero runtime dependencies.

## Features

- **Multi-database** — MySQL/MariaDB, PostgreSQL, and SQLite from one binary
- **6 MCP tools** — `list_databases`, `list_tables`, `get_table_schema`, `get_table_schema_with_relations`, `execute_sql`, `create_database`
- **Single binary** — ~7 MB, no Python/Node/Docker needed
- **Multiple transports** — stdio (for Claude Desktop, Cursor) and HTTP (for remote/multi-client)
- **SSL/TLS** — configurable certificates for MySQL and PostgreSQL

## Configuration

All settings via environment variables or `.env` file. Copy `.env.example` to get started.

### Database Connection

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_HOST` | `localhost` | Database host |
| `DB_PORT` | `3306` | Database port |
| `DB_USER` | *(required)* | Database user |
| `DB_PASSWORD` | *(required)* | Database password |
| `DB_NAME` | *(none)* | Default database |

For SQLite, use `--db-path ./file.db` instead — no host/user/password needed.

### Server Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `MCP_READ_ONLY` | `true` | Block write queries |
| `MCP_MAX_POOL_SIZE` | `10` | Max connection pool size |
| `LOG_LEVEL` | `info` | Log level (trace/debug/info/warn/error) |
| `LOG_FILE` | `logs/mcp_server.log` | Log file path |

### SSL/TLS (MySQL/PostgreSQL)

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_SSL` | `false` | Enable SSL |
| `DB_SSL_CA` | *(none)* | CA certificate path |
| `DB_SSL_CERT` | *(none)* | Client certificate path |
| `DB_SSL_KEY` | *(none)* | Client key path |
| `DB_SSL_VERIFY_CERT` | `true` | Verify server certificate |
| `DB_SSL_VERIFY_IDENTITY` | `false` | Verify server hostname |

### HTTP Transport

| Variable | Default | Description |
|----------|---------|-------------|
| `ALLOWED_ORIGINS` | `http://localhost,...` | CORS allowed origins (comma-separated) |
| `ALLOWED_HOSTS` | `localhost,127.0.0.1` | Trusted Host headers (comma-separated) |

## CLI Reference

```
sql-mcp [OPTIONS]

Options:
  --database-type <TYPE>  Database type: mysql, postgres, sqlite [default: mysql]
  --db-path <PATH>        SQLite database file path (required for sqlite)
  --transport <MODE>      Transport mode: stdio, http [default: stdio]
  --host <HOST>           Bind host for HTTP transport [default: 127.0.0.1]
  --port <PORT>           Bind port for HTTP transport [default: 9001]
  -h, --help              Print help
  -V, --version           Print version
```

## MCP Tools

### list_databases

Lists all accessible databases. Returns a JSON array of database names.

### list_tables

Lists all tables in a database. Parameters: `database_name`.

### get_table_schema

Returns column definitions (type, nullable, key, default, extra) for a table. Parameters: `database_name`, `table_name`.

### get_table_schema_with_relations

Same as `get_table_schema` plus foreign key relationships (constraint name, referenced table/column, on update/delete rules). Parameters: `database_name`, `table_name`.

### execute_sql

Executes a SQL query. In read-only mode (default), only SELECT, SHOW, DESCRIBE, and USE are allowed. Parameters: `sql_query`, `database_name`.

### create_database

Creates a database if it doesn't exist. Blocked in read-only mode. Not supported for SQLite. Parameters: `database_name`.

## Security

- **Read-only mode** (default) — AST-based SQL parsing validates every query before execution
- **Single-statement enforcement** — multi-statement injection blocked at parse level
- **Dangerous function blocking** — `LOAD_FILE()`, `INTO OUTFILE`, `INTO DUMPFILE` detected in the AST
- **Identifier validation** — database/table names restricted to alphanumeric + underscore
- **CORS + trusted hosts** — configurable for HTTP transport
- **SSL/TLS** — encrypted connections with certificate verification

## Testing

```bash
# Run all tests
cargo test

# With MCP Inspector
npx @modelcontextprotocol/inspector ./target/release/sql-mcp

# HTTP mode testing
curl -X POST http://localhost:9001/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}'
```

## Development

```bash
cargo build              # Development build
cargo build --release    # Release build (~7 MB)
cargo test               # Run tests
cargo clippy -- -D warnings  # Lint
cargo fmt                # Format
cargo doc --no-deps      # Build documentation
```
