use quick_xml::events::Event;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::rave::context::{DataDictionaryEntry, ParseContext};

/// Parse DataDictionaries and DataDictionaryEntries worksheets.
pub fn parse_data_dictionaries<R: std::io::Read>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    // Parse DataDictionaryEntries worksheet
    parse_dictionary_entries(reader, context)
}

/// Parse DataDictionaryEntries worksheet into context
fn parse_dictionary_entries<R: std::io::Read>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut buffer = Vec::new();
    let mut in_entry = false;

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) if e.name().as_ref() == b"Row" => {
                // Start of a row - could be header or data
                in_entry = true;
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"Row" => {
                in_entry = false;
            }
            Ok(Event::Text(e)) if in_entry => {
                let text = e.unescape().map_err(|e| AlsParseError::XmlError(e.to_string()))?;
                // Parse tab-separated row data
                let fields: Vec<&str> = text.split('\t').collect();
                if fields.len() >= 4 {
                    // DataDictionaryName, CodedData, Ordinal, UserDataString, Specify
                    let dictionary_name = fields[0].to_string();
                    let coded_data = fields[1].to_string();
                    let ordinal = fields[2].parse::<i32>().unwrap_or(0);
                    let user_data_string = fields[3].to_string();
                    let specify = fields.get(4).map(|s| s == "TRUE").unwrap_or(false);

                    context.add_dictionary_entry(DataDictionaryEntry {
                        dictionary_name,
                        coded_data,
                        ordinal,
                        user_data_string,
                        specify,
                    });
                }
            }
            Ok(_) => {}
            Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
        }
    }

    Ok(())
}