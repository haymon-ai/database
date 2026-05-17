//! Shared MCP server utilities and request types.
//!
//! Provides [`types`] for tool request/response schemas,
//! [`pagination`] cursor helpers, the [`registry`] tool table, and the
//! [`Server`] wrapper plus [`server_info`] used by per-backend servers.

pub mod pagination;
pub mod registry;
mod server;
pub mod types;

pub use pagination::{Cursor, Pager};
pub use registry::{ToolSpec, build_tool_router};
pub use server::{Server, server_info};
