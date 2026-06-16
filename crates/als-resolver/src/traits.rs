use crate::error::AlsParseError;
use entities::project::Project;
use std::path::Path;

/// Parser trait for ALS (Audit Landmark Study) files.
/// Implementors parse different ALS formats (Rave, ecollect, etc.)
/// into a unified Project structure.
pub trait AlsParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError>;
}
