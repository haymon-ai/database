//! Declarative tool registry for per-backend MCP routers.
//!
//! A backend declares its tools as a `const` slice of [`ToolSpec`] rows, each
//! row carrying the gating flags for that tool. [`build_tool_router`] folds the
//! slice into a [`ToolRouter`], skipping tools that the current mode forbids.
//! This keeps the read-only / pinned gating matrix as data, not control flow.

use rmcp::handler::server::router::tool::{AsyncTool, ToolRouter};

/// Declarative registration entry for one MCP tool.
///
/// Pairs the tool's router-registration function with its mode gates.
pub struct ToolSpec<H: Send + Sync + 'static> {
    /// Registers the tool on a router, returning the extended router.
    register: fn(ToolRouter<H>) -> ToolRouter<H>,
    /// Whether the tool is hidden in read-only mode.
    read_only: bool,
    /// Whether the tool is hidden when a database name is pinned.
    pinned: bool,
}

impl<H: Send + Sync + 'static> std::fmt::Debug for ToolSpec<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolSpec")
            .field("read_only", &self.read_only)
            .field("pinned", &self.pinned)
            .finish_non_exhaustive()
    }
}

impl<H: Send + Sync + 'static> ToolSpec<H> {
    /// Creates a spec for tool `T` with its read-only and pinned gates.
    #[must_use]
    pub const fn new<T: AsyncTool<H> + 'static>(read_only: bool, pinned: bool) -> Self {
        Self {
            register: ToolRouter::with_async_tool::<T>,
            read_only,
            pinned,
        }
    }
}

/// Builds a [`ToolRouter`] from `specs`, skipping mode-gated tools.
///
/// Write tools are skipped when `read_only`; cross-database tools when
/// `pinned`.
#[must_use]
pub fn build_tool_router<H: Send + Sync + 'static>(
    specs: &[ToolSpec<H>],
    read_only: bool,
    pinned: bool,
) -> ToolRouter<H> {
    specs
        .iter()
        .filter(|spec| (!spec.read_only || !read_only) && (!spec.pinned || !pinned))
        .fold(ToolRouter::new(), |router, spec| (spec.register)(router))
}
