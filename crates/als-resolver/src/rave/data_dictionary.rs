use crate::error::AlsParseError;
use crate::rave::context::{DataDictionaryEntry, ParseContext};
use crate::rave::sheet::SheetRows;
use quick_xml::Reader;

/// Parse the DataDictionaryEntries worksheet into the context.
///
/// The sheet holds one row per coded value: the dictionary it belongs to, the
/// code itself, its ordinal and the label shown to the user.
pub fn parse_data_dictionaries<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut is_header = true;

    for row in SheetRows::new(reader) {
        let row = row?;
        if is_header {
            is_header = false;
            continue;
        }
        if row.len() < 4 {
            continue;
        }

        let dictionary_name = row[0].clone();
        if dictionary_name.is_empty() || dictionary_name == "DataDictionaryName" {
            continue;
        }

        let coded_data = row[1].clone();
        let ordinal = row[2].parse::<i32>().unwrap_or(0);
        let user_data_string = row[3].clone();
        let specify = row.get(4).map(|s| s.as_str()) == Some("TRUE");

        context.add_dictionary_entry(DataDictionaryEntry {
            dictionary_name,
            coded_data,
            ordinal,
            user_data_string,
            specify,
        });
    }

    Ok(())
}
