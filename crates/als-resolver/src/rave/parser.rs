use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::data_dictionary::parse_data_dictionaries;
use crate::rave::fields::parse_fields;
use crate::rave::folders::parse_folders;
use crate::rave::forms::parse_forms;
use crate::rave::matrices::{find_study_matrix_sheet, parse_matrix_master};
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
        {
            let mut reader = open_sheet(&bytes, "DataDictionaryEntries")?;
            parse_data_dictionaries(&mut reader, &mut context)?;
        }

        // Phase 2: Parse Forms
        {
            let mut reader = open_sheet(&bytes, "Forms")?;
            parse_forms(&mut reader, &mut context)?;
        }

        // Phase 3: Parse Fields
        {
            let mut reader = open_sheet(&bytes, "Fields")?;
            parse_fields(&mut reader, &mut context)?;
        }

        // Phase 4: Parse Folders to create Visit structs
        {
            let mut reader = open_sheet(&bytes, "Folders")?;
            parse_folders(&mut reader, &mut context)?;
        }

        // Phase 5: Parse the study matrix to populate Visit.forms
        if let Some(matrix_sheet) = find_study_matrix(&bytes)? {
            let mut reader = open_sheet(&bytes, &matrix_sheet)?;
            parse_matrix_master(&mut reader, &mut context)?;
        }

        let mut forms: Vec<_> = context.forms.into_values().collect();
        forms.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.name.cmp(&b.name)));

        let mut visits = context.visits;
        visits.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.code.cmp(&b.code)));

        Ok(Project {
            forms,
            visit: visits,
        })
    }
}

/// Open the reader on the given worksheet of the workbook.
fn open_sheet<'a>(
    bytes: &'a [u8],
    worksheet_name: &str,
) -> Result<Reader<&'a [u8]>, AlsParseError> {
    let mut reader = Reader::from_reader(bytes);
    // `<Cell/>` and `<Data/>` are written as empty elements, expanding them
    // keeps the start/end events of every cell balanced.
    reader.config_mut().expand_empty_elements = true;
    navigate_to_worksheet(&mut reader, worksheet_name)?;
    Ok(reader)
}

/// Find the worksheet holding the study wide matrix, if the workbook has one.
fn find_study_matrix(bytes: &[u8]) -> Result<Option<String>, AlsParseError> {
    let mut reader = match open_sheet(bytes, "Matrices") {
        Ok(reader) => reader,
        // A workbook without matrices still holds forms and visits.
        Err(AlsParseError::WorksheetNotFound(_)) => return Ok(None),
        Err(e) => return Err(e),
    };
    find_study_matrix_sheet(&mut reader)
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
