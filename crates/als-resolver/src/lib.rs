pub mod ecollect_v6;
mod error;
mod rave;
mod traits;

pub use entities::project::Project;
pub use error::AlsParseError;
pub use traits::AlsParser;

use std::path::Path;

/// Parse a Rave ALS file from a path.
pub fn parse_rave_als(path: &Path) -> Result<Project, AlsParseError> {
    rave::parser::RaveParser.parse(path)
}

/// Parse an ecollect v6 ALS file from a path.
pub fn parse_ecollect_v6_als(path: &Path) -> Result<Project, AlsParseError> {
    crate::ecollect_v6::EcollectV6Parser.parse(path)
}
