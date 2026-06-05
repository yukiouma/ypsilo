use quick_xml::events::Event;
use quick_xml::escape::unescape;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::rave::context::{FormRow, ParseContext};
use entities::project::CRFForm;

/// Parse the Forms worksheet (stop at worksheet boundary).
pub fn parse_forms<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut buffer = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_data_cell = false;
    let mut current_cell_index = 0;
    let mut row_count = 0;

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => {
                break;
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"Worksheet" => {
                break;
            }
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Row" => {
                        current_row.clear();
                        current_cell_index = 0;
                        row_count += 1;
                    }
                    b"Cell" => {
                        // Check for ss:Index attribute to handle skipped columns
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"ss:Index" {
                                if let Ok(idx_str) = std::str::from_utf8(attr.value.as_ref()) {
                                    if let Ok(idx) = idx_str.parse::<usize>() {
                                        // Pad current_row with empty strings up to idx - 1
                                        while current_row.len() < idx {
                                            current_row.push(String::new());
                                        }
                                        current_cell_index = idx;
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
                        // Process completed row
                        if current_row.len() >= 16 && row_count > 1 {
                            let oid = current_row[0].clone();
                            let ordinal = current_row[1].parse::<i32>().unwrap_or(0);
                            let draft_form_name = current_row[2].clone();

                            if !oid.is_empty() && oid != "OID" {
                                context.form_rows.push(FormRow {
                                    oid: oid.clone(),
                                    ordinal,
                                    draft_form_name: draft_form_name.clone(),
                                    link_folder_oid: current_row.get(14).cloned(),
                                });

                                context.forms.insert(
                                    oid.clone(),
                                    CRFForm {
                                        name: oid,
                                        description: draft_form_name,
                                        order: ordinal,
                                        items: Vec::new(),
                                        domains: Vec::new(),
                                        annotations: Vec::new(),
                                    },
                                );
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
                    // Ensure the row has enough slots
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