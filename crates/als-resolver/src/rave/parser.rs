use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::data_dictionary::parse_data_dictionaries;
use crate::rave::fields::parse_fields;
use crate::rave::folders::parse_folders;
use crate::rave::forms::parse_forms;
use crate::rave::matrices::{find_master_matrix_sheet, parse_matrix_master};
use crate::traits::AlsParser;
use entities::project::Project;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

/// Rave ALS parser implementation.
pub struct RaveParser;

impl AlsParser for RaveParser {
    fn parse(&self, path: &std::path::Path) -> Result<Project, AlsParseError> {
        let file = std::fs::File::open(path).map_err(AlsParseError::IoError)?;
        self.parse_reader(std::io::BufReader::new(file))
    }

    fn parse_reader(&self, mut reader: impl std::io::Read) -> Result<Project, AlsParseError> {
        let mut context = ParseContext::new();
        // Read entire file into memory to allow multiple passes
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;

        // Phase 1: Load DataDictionaries
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

        // Find out the name of MASTER sheet, the sheet name will be something like "Matrix121#MASTER"
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Matrices")?;
        let master_sheet_name = find_master_matrix_sheet(&mut reader)?;

        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, &master_sheet_name)?;
        parse_matrix_master(&mut reader, &mut context)?;

        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: context.visits,
        })
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
