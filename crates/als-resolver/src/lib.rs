mod error;
mod traits;
mod rave;

pub use error::AlsParseError;
pub use traits::AlsParser;
pub use entities::project::Project;

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Parse a Rave ALS file from a path.
pub fn parse_rave_als(path: &Path) -> Result<Project, AlsParseError> {
    let file = File::open(path).map_err(|e| AlsParseError::IoError(e.to_string()))?;
    parse_rave_als_stream(file)
}

/// Parse a Rave ALS file from any Read source.
pub fn parse_rave_als_stream(input: impl Read + 'static) -> Result<Project, AlsParseError> {
    rave::parser::RaveParser.parse(input)
}