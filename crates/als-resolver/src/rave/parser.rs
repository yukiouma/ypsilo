use std::io::Read;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::traits::AlsParser;
use crate::rave::context::ParseContext;
use crate::rave::data_dictionary::parse_data_dictionaries;
use crate::rave::forms::parse_forms;
use crate::rave::fields::parse_fields;
use crate::rave::matrices::parse_matrices;
use entities::project::Project;

/// Rave ALS parser implementation.
pub struct RaveParser;

impl AlsParser for RaveParser {
    fn parse(self, source: impl Read + 'static) -> Result<Project, AlsParseError> {
        let mut context = ParseContext::new();
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);

        // Phase 1: Load DataDictionaries
        // Navigate to DataDictionaryEntries worksheet
        navigate_to_worksheet(&mut reader, "DataDictionaryEntries")?;
        parse_data_dictionaries(&mut reader, &mut context)?;

        // Phase 2: Parse Forms
        navigate_to_worksheet(&mut reader, "Forms")?;
        parse_forms(&mut reader, &mut context)?;

        // Phase 3: Parse Fields
        navigate_to_worksheet(&mut reader, "Fields")?;
        parse_fields(&mut reader, &mut context)?;

        // Phase 4: Parse Folders (placeholder - no-op for now)
        // navigate_to_worksheet(&mut reader, "Folders")?;

        // Phase 5: Parse Matrices
        navigate_to_worksheet(&mut reader, "Matrices")?;
        parse_matrices(&mut reader, &mut context)?;

        // Build and return Project
        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: context.visits,
        })
    }
}

/// Navigate to a worksheet by name.
fn navigate_to_worksheet<R: Read>(
    reader: &mut Reader<R>,
    worksheet_name: &str,
) -> Result<(), AlsParseError> {
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => {
                return Err(AlsParseError::WorksheetNotFound(worksheet_name.to_string()));
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"Worksheet" => {
                // Check if this is the worksheet we want
                let mut is_target = false;
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"ss:Name" || attr.key.as_ref() == b"Name" {
                        if attr.value.as_ref() == worksheet_name.as_bytes() {
                            is_target = true;
                            break;
                        }
                    }
                }
                if is_target {
                    return Ok(());
                }
            }
            Ok(_) => {}
            Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
        }
    }
}

use quick_xml::events::Event;