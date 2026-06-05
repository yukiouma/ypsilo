use quick_xml::events::Event;
use quick_xml::escape::unescape;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use entities::project::Visit;

/// Parse Matrices worksheet and Matrix sheets to extract visits.
pub fn parse_matrices<R: std::io::BufRead>(
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
                        // Process Matrix row (skip header row)
                        if current_row.len() >= 3 && current_row[0] != "MatrixName" {
                            let matrix_name = current_row[0].clone();
                            let oid = current_row[1].clone();
                            let _maximum = current_row[2].parse::<i32>().unwrap_or(0);

                            if !oid.is_empty() {
                                let visit = Visit {
                                    code: oid.clone(),
                                    name: matrix_name,
                                    order: context.visits.len() as i32 + 1,
                                    forms: Vec::new(),
                                };
                                context.visits.push(visit);
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

/// Parse a Matrix sheet (e.g., Matrix1#C1) to extract form bindings.
/// This extracts form OIDs from the first column ("Matrix: {OID}").
pub fn parse_matrix_sheet<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    visit_code: &str,
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
                        // First column contains the form OID
                        if let Some(form_oid) = current_row.first() {
                            if !form_oid.is_empty() && form_oid != "Matrix: {OID}" && form_oid != "Subject" {
                                // Find the visit and add form OID
                                if let Some(visit) = context.visits.iter_mut().find(|v| v.code == visit_code) {
                                    if !visit.forms.contains(form_oid) {
                                        visit.forms.push(form_oid.clone());
                                    }
                                }
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