// ecollect_legacy parser module

use crate::AlsParseError;
use crate::AlsParser;
use crate::Project;
use std::path::Path;

pub struct EcollectLegacyParser;

impl AlsParser for EcollectLegacyParser {
    fn parse(&self, _path: &Path) -> Result<Project, AlsParseError> {
        todo!("EcollectLegacyParser not yet implemented")
    }
}
