use crate::error::AlsParseError;
use entities::project::Project;

/// Parser trait for ALS (Audit Landmark Study) files.
/// Implementors parse different ALS formats (Rave, ecollect, etc.)
/// into a unified Project structure.
pub trait AlsParser {
    fn parse(self, source: impl std::io::Read + 'static) -> Result<Project, AlsParseError>;
}
