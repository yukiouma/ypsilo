use quick_xml::events::Event;
use quick_xml::escape::unescape;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::rave::context::{FormRow, ParseContext};
use entities::project::CRFForm;

/// Parse the Forms worksheet.
pub fn parse_forms<R: std::io::BufRead>(
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
                        // Process completed row
                        if current_row.len() >= 16 {
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
                    current_row.push(text.to_string());
                }
            }
            Ok(_) => {}
            Err(e) => return Err(AlsParseError::XmlError(e.to_string())),
        }
    }

    Ok(())
}