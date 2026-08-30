use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use entities::project::{CRFItem, ControlType};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;

/// Parse the Fields worksheet and populate form items (stop at worksheet boundary).
pub fn parse_fields<R: std::io::BufRead>(
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
                        if current_row.len() >= 37 && current_row[0] != "FormOID" {
                            let form_oid = current_row[0].clone();
                            let field_oid = current_row[1].clone();
                            let _ordinal = current_row[2].parse::<i32>().unwrap_or(0);
                            let draft_field_name = current_row[4].clone();
                            let variable_oid = current_row[6].clone();
                            let data_format = current_row[7].clone();
                            let data_dictionary_name = current_row[8].clone();
                            let control_type_str = current_row[11].clone();
                            let pre_text = current_row[14].clone();
                            let fixed_unit = current_row[15].clone();

                            // Get options from DataDictionary if present
                            let item_option = if !data_dictionary_name.is_empty() {
                                let options = context.get_options(&data_dictionary_name);
                                if options.is_empty() {
                                    None
                                } else {
                                    Some(options)
                                }
                            } else {
                                None
                            };

                            // Map control type string to ControlType enum.
                            // The keys here are the literal strings produced by RAVE in the
                            // Fields worksheet; anything unrecognized falls back to TEXT.
                            let control_type = match control_type_str.as_str() {
                                "DateTime" => ControlType::DATETIME,
                                "CheckBox" => ControlType::CHECKBOX,
                                "DropDownList" => ControlType::SELECTION,
                                "RadioButton" => ControlType::SELECTION,
                                "RadioButton (Vertical)" => ControlType::SELECTION,
                                _ => ControlType::TEXT,
                            };

                            // Use PreText as label if available, otherwise DraftFieldName
                            let label = if !pre_text.is_empty() {
                                pre_text
                            } else if !draft_field_name.is_empty() {
                                draft_field_name
                            } else {
                                "".into()
                            };

                            let item_unit = if !fixed_unit.is_empty() {
                                Some(entities::project::ItemUnit {
                                    value: fixed_unit,
                                    annotations: Vec::new(),
                                })
                            } else {
                                None
                            };

                            let item = CRFItem {
                                name: field_oid,
                                label,
                                item_option,
                                annotations: Vec::new(),
                                format: data_format,
                                control_type,
                                item_unit,
                                not_variable: if variable_oid.is_empty() {
                                    Some(true)
                                } else {
                                    None
                                },
                            };

                            // Add item to the corresponding form
                            if let Some(form) = context.forms.get_mut(&form_oid) {
                                form.items.push(item);
                            }
                        }
                        current_row.clear();
                    }
                    b"Data" => {
                        // End of one <Data> element: advance to the next cell.
                        // Cell text (including any split by entity references) is now
                        // complete in current_row[current_cell_index]. Pad with empty
                        // strings so that cells with no Text event (e.g. <Data/>) still
                        // occupy a slot in the row.
                        in_data_cell = false;
                        while current_row.len() <= current_cell_index {
                            current_row.push(String::new());
                        }
                        current_cell_index += 1;
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
                    // Append to the current cell instead of overwriting. quick-xml
                    // splits text content at entity references, so a single <Data>
                    // element containing `&gt;` etc. emits multiple Text events;
                    // they all belong to the same cell.
                    current_row[current_cell_index].push_str(&text);
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if in_data_cell {
                    // quick-xml emits a GeneralRef event for each XML entity
                    // reference inside text content (e.g. `&gt;` -> "gt"). Decode
                    // it back to its character so it joins the surrounding text.
                    let entity_name =
                        std::str::from_utf8(e.as_ref()).map_err(|err| AlsParseError::XmlError(err.to_string()))?;
                    let decoded_char = match entity_name {
                        "amp" => '&',
                        "lt" => '<',
                        "gt" => '>',
                        "quot" => '"',
                        "apos" => '\'',
                        other => {
                            // Unknown/numeric entity: best-effort fallback. quick-xml
                            // represents numeric references with their resolved char
                            // in the surrounding Text event, so reaching here means
                            // the entity is unsupported. Drop the reference rather
                            // than inject a stray `&`.
                            let _ = other;
                            return Err(AlsParseError::XmlError(format!(
                                "Unsupported XML entity reference: &{other};"
                            )));
                        }
                    };
                    while current_row.len() <= current_cell_index {
                        current_row.push(String::new());
                    }
                    current_row[current_cell_index].push(decoded_char);
                }
            }
            Ok(_) => {}
            Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities::project::CRFForm;

    /// Build a Fields row of at least 37 cells (the minimum the parser accepts)
    /// where every cell is empty except for the columns the parser reads.
    fn build_row_xml(pre_text: &str, form_oid: &str, field_oid: &str) -> String {
        // 37 cells; every cell has a <Data> element (matches real SpreadsheetML).
        // Indices: 0 FormOID, 1 FieldOID, 2 Ordinal, 4 DraftFieldName,
        //          6 VariableOID, 7 DataFormat, 8 DataDictionaryName,
        //          11 ControlType, 14 PreText, 15 FixedUnit
        let mut cells: Vec<String> = (0..37).map(|_| String::from("<Cell><Data></Data></Cell>")).collect();
        cells[0] = format!("<Cell><Data>{form_oid}</Data></Cell>");
        cells[1] = format!("<Cell><Data>{field_oid}</Data></Cell>");
        cells[2] = String::from("<Cell><Data>1</Data></Cell>");
        cells[4] = format!("<Cell><Data>{field_oid}</Data></Cell>");
        cells[11] = String::from("<Cell><Data>Text</Data></Cell>");
        // Escape the pre_text for safe XML embedding.
        let escaped = pre_text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        cells[14] = format!("<Cell><Data>{escaped}</Data></Cell>");
        format!("<Row>{}</Row>", cells.join(""))
    }

    fn make_context(form_oid: &str) -> ParseContext {
        let mut context = ParseContext::new();
        context.forms.insert(
            form_oid.to_string(),
            CRFForm {
                name: form_oid.to_string(),
                description: form_oid.to_string(),
                order: 0,
                items: Vec::new(),
                domains: Vec::new(),
                annotations: Vec::new(),
            },
        );
        context
    }

    #[test]
    fn cell_with_entity_reference_is_not_split() {
        // This is the exact scenario from the bug report: a PreText cell whose
        // raw value contains `&gt;` must be parsed as a single, intact string.
        let pre_text = "Infusion-related reactions (grade>= 3)";
        let xml = build_row_xml(pre_text, "SV", "SVFOO");
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);
        let mut context = make_context("SV");

        parse_fields(&mut reader, &mut context).unwrap();

        let form = context.forms.get("SV").expect("form should exist");
        assert_eq!(form.items.len(), 1, "exactly one item should be parsed");
        assert_eq!(form.items[0].label, pre_text);
    }

    #[test]
    fn cell_without_entity_reference_still_parses() {
        // Regression check: cells with plain text must still end up in the
        // correct slot after the cell-indexing change.
        let xml = build_row_xml("Visit Date", "SV", "SVDAT");
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);
        let mut context = make_context("SV");

        parse_fields(&mut reader, &mut context).unwrap();

        let form = context.forms.get("SV").unwrap();
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].label, "Visit Date");
        assert_eq!(form.items[0].name, "SVDAT");
    }
}