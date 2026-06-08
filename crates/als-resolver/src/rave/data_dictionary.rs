use crate::error::AlsParseError;
use crate::rave::context::{DataDictionaryEntry, ParseContext};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;

/// Parse DataDictionaries and DataDictionaryEntries worksheets.
pub fn parse_data_dictionaries<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    // Parse DataDictionaryEntries worksheet
    parse_dictionary_entries(reader, context)
}

/// Parse DataDictionaryEntries worksheet into context (stop at worksheet boundary)
fn parse_dictionary_entries<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut buffer = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_data_cell = false;
    let mut current_cell_index = 0;

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::End(e)) if e.name().as_ref() == b"Worksheet" => {
                break;
            }
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Row" => {
                        current_row.clear();
                        current_cell_index = 0;
                    }
                    b"Cell" => {
                        // Check for ss:Index attribute to handle skipped columns
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"ss:Index" {
                                if let Ok(idx_str) = std::str::from_utf8(attr.value.as_ref()) {
                                    if let Ok(idx) = idx_str.parse::<usize>() {
                                        while current_row.len() < idx {
                                            current_row.push(String::new());
                                        }
                                        current_cell_index = idx - 1;
                                    }
                                }
                            }
                        }
                    }
                    b"Data" => {
                        in_data_cell = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Row" => {
                        // Process completed row (skip header row)
                        if current_row.len() >= 4 && current_row[0] != "DataDictionaryName" {
                            let dictionary_name = current_row[0].clone();
                            let coded_data = current_row[1].clone();
                            let ordinal = current_row[2].parse::<i32>().unwrap_or(0);
                            let user_data_string = current_row[3].clone();
                            let specify = current_row.get(4).map(|s| s.as_str()) == Some("TRUE");

                            context.add_dictionary_entry(DataDictionaryEntry {
                                dictionary_name,
                                coded_data,
                                ordinal,
                                user_data_string,
                                specify,
                            });
                        }
                        current_row.clear();
                    }
                    b"Data" => {
                        in_data_cell = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_data_cell {
                    let decoded = e
                        .decode()
                        .map_err(|e| AlsParseError::XmlError(e.to_string()))?;
                    let text =
                        unescape(&decoded).map_err(|e| AlsParseError::XmlError(e.to_string()))?;
                    while current_row.len() <= current_cell_index {
                        current_row.push(String::new());
                    }
                    current_row[current_cell_index] = text.to_string();
                    current_cell_index += 1;
                }
            }
            Ok(_) => {}
            Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
        }
    }

    Ok(())
}
