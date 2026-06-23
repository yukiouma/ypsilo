use std::io::{Read, Seek};
use std::path::Path;

use crate::error::AlsParseError;
use entities::project::Project;

/// Parser trait for ALS (Audit Landmark Study) files.
/// Implementors parse different ALS formats (Rave, ecollect, etc.)
/// into a unified Project structure.
pub trait AlsParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let file = std::fs::File::open(path).map_err(AlsParseError::IoError)?;
        self.parse_reader(std::io::BufReader::new(file))
    }

    fn parse_reader(&self, reader: impl Read + Seek) -> Result<Project, AlsParseError>;
}
