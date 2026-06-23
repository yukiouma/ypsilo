use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::data_dictionary::parse_data_dictionaries;
use crate::rave::fields::parse_fields;
use crate::rave::forms::parse_forms;
use crate::rave::folders::parse_folders;
use crate::rave::matrices::parse_matrix_master;
use crate::traits::AlsParser;
use entities::project::Project;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::Path;

/// Rave ALS parser implementation.
pub struct RaveParser;

impl AlsParser for RaveParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let mut context = ParseContext::new();
        // Read entire file into memory to allow multiple passes
        let mut file = File::open(path).map_err(AlsParseError::IoError)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Phase 1: Load DataDictionaries
        // Navigate to DataDictionaryEntries worksheet
        let mut reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "DataDictionaryEntries")?;
        parse_data_dictionaries(&mut reader, &mut context)?;

        // Phase 2: Parse Forms
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Forms")?;
        parse_forms(&mut reader, &mut context)?;

        // Phase 3: Parse Fields
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Fields")?;
        parse_fields(&mut reader, &mut context)?;

        // Phase 4: Parse Folders to create Visit structs
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Folders")?;
        parse_folders(&mut reader, &mut context)?;

        // Phase 5: Parse Matrix#MASTER to populate Visit.forms
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Matrix121#MASTER")?;
        parse_matrix_master(&mut reader, &mut context)?;

        // Build and return Project
        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: context.visits,
        })
    }

    fn parse_reader(&self, _reader: impl Read) -> Result<Project, AlsParseError> {
        // Rave reads Excel files which require file-based access for worksheet navigation
        Err(AlsParseError::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "rave does not support reader-based parsing",
        )))
    }
}

/// Navigate to a worksheet by name.
fn navigate_to_worksheet<R: BufRead>(
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
