use quick_xml::events::Event;
use quick_xml::escape::unescape;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use entities::project::{CRFItem, ControlType};

/// Parse the Fields worksheet and populate form items.
pub fn parse_fields<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut buffer = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_data_cell = false;

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Row" => {
                        current_row.clear();
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
                            let ordinal = current_row[2].parse::<i32>().unwrap_or(0);
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

                            // Map control type string to ControlType enum
                            let control_type = match control_type_str.as_str() {
                                "Text" => ControlType::TEXT,
                                "Select" => ControlType::SELECTION,
                                "Check" => ControlType::CHECKBOX,
                                "Radio" => ControlType::SELECTION,
                                "File" => ControlType::TEXT,
                                _ => ControlType::TEXT,
                            };

                            // Use PreText as label if available, otherwise DraftFieldName
                            let label = if !pre_text.is_empty() {
                                pre_text
                            } else if !draft_field_name.is_empty() {
                                draft_field_name
                            } else {
                                variable_oid
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
                                not_variable: None,
                            };

                            // Add item to the corresponding form
                            if let Some(form) = context.forms.get_mut(&form_oid) {
                                form.items.push(item);
                            }
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
                    let decoded = e.decode().map_err(|e| AlsParseError::XmlError(e.to_string()))?;
                    let text = unescape(&decoded).map_err(|e| AlsParseError::XmlError(e.to_string()))?;
                    current_row.push(text.to_string());
                }
            }
            Ok(_) => {}
            Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
        }
    }

    Ok(())
}