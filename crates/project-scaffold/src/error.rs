use std::{io, path::PathBuf};
use thiserror::Error;

/// Every failure mode of the project scaffold crate.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse config at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("unknown template variable `{{{{{key}}}}}` in `{template}`")]
    UnknownTemplateKey { key: String, template: String },

    #[error("root path does not exist: {0}")]
    RootPathNotFound(PathBuf),

    #[error("project root does not exist: {0}")]
    ProjectRootNotFound(PathBuf),

    #[error("config file does not exist: {0}")]
    ConfigPathNotFound(PathBuf),

    #[error("reference file does not exist: {0}")]
    ReferenceNotFound(PathBuf),

    #[error("migration directory does not exist: {0}")]
    MigrationDirNotFound(PathBuf),
}

impl ScaffoldError {
    /// Wrap an [`io::Error`] together with the path it occurred on, so failures deep
    /// in the tree stay identifiable.
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
