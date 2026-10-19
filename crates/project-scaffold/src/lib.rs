//! Scaffold the directory tree of a clinical trial data processing project.

mod config;
mod context;
mod error;
mod scaffold;
mod template;

pub use config::{Migration, Node, ProjectScaffoldConfig};
pub use context::{Context, RawContext};
pub use error::ScaffoldError;

/// Result type for every fallible operation in this crate.
pub type Result<T> = std::result::Result<T, ScaffoldError>;

/// Materialise the project scaffold described by `config`.
///
/// Every path in `config` has already had its `{{ ... }}` placeholders resolved by
/// [`ProjectScaffoldConfig::build`]. Paths that already exist are left untouched, so
/// calling this twice over the same project root is a no-op.
pub fn project_scaffolding(config: ProjectScaffoldConfig) -> Result<()> {
    scaffold::build_tree(&config.directories, config.project_root.as_ref())
}
