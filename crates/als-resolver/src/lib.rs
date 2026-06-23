pub mod ecollect_v6;
mod error;
mod rave;
mod traits;

pub use entities::project::Project;
pub use error::AlsParseError;
pub use traits::AlsParser;

use std::io::{Read, Seek};

pub mod ecollect_legacy;

/// Parse a Rave ALS file from any `impl Read + Seek` source.
pub fn parse_rave_als(reader: impl Read + Seek) -> Result<Project, AlsParseError> {
    rave::parser::RaveParser.parse_reader(reader)
}

/// Parse an ecollect v6 ALS file from any `impl Read + Seek` source.
pub fn parse_ecollect_v6_als(reader: impl Read + Seek) -> Result<Project, AlsParseError> {
    crate::ecollect_v6::EcollectV6Parser.parse_reader(reader)
}

/// Parse an ecollect legacy ALS file from any `impl Read + Seek` source.
pub fn parse_ecollect_legacy_als(reader: impl Read + Seek) -> Result<Project, AlsParseError> {
    crate::ecollect_legacy::EcollectLegacyParser.parse_reader(reader)
}
