use quick_xml::events::Event;
use quick_xml::escape::unescape;
use quick_xml::Reader;
use crate::error::AlsParseError;
use crate::rave::context::ParseContext;

/// Parse the Matrix#MASTER sheet to populate Visit.forms.
/// The MASTER sheet has columns for each visit (SCR, C1, C2, ...) and rows for forms.
/// "X" in a cell indicates the form is bound to that visit.
pub fn parse_matrix_master<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut buffer = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_data_cell = false;
    let mut current_cell_index = 0;
    let mut visit_codes: Vec<String> = Vec::new(); // Column index -> visit code
    let mut row_count = 0;

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::End(e)) if e.name().as_ref() == b"Worksheet" => break,
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Row" => {
                        current_row.clear();
                        current_cell_index = 0;
                        row_count += 1;
                    }
                    b"Cell" => {
                        let mut found = false;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"ss:Index" {
                                if let Ok(idx_str) = std::str::from_utf8(attr.value.as_ref()) {
                                    if let Ok(idx) = idx_str.parse::<usize>() {
                                        // ss:Index is 1-based, convert to 0-based for array index
                                        let array_idx = idx.saturating_sub(1);
                                        while current_row.len() < array_idx {
                                            current_row.push(String::new());
                                        }
                                        current_cell_index = array_idx;
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !found {
                            // No ss:Index, use current position and increment for next
                            while current_row.len() <= current_cell_index {
                                current_row.push(String::new());
                            }
                            // Don't increment here - Text event will handle increment
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
                        if row_count == 1 {
                            // Header row
                            visit_codes.clear();
                            for (i, val) in current_row.iter().enumerate() {
                                if i >= 2 && !val.is_empty() {
                                    visit_codes.push(val.clone());
                                }
                            }
                        } else if row_count > 1 && !current_row.is_empty() {
                            // Data row: first column is form OID, "X" marks bound visits
                            let form_oid = current_row[0].clone();
                            if !form_oid.is_empty() && form_oid != "Matrix: MASTER" && form_oid != "Subject" {
                                for (col_idx, val) in current_row.iter().enumerate() {
                                    if col_idx >= 2 && val == "X" {
                                        let visit_idx = col_idx.saturating_sub(2);
                                        if visit_idx < visit_codes.len() {
                                            let visit_code = &visit_codes[visit_idx];
                                            if let Some(visit) = context.visits.iter_mut().find(|v| v.code == *visit_code) {
                                                if !visit.forms.contains(&form_oid) {
                                                    visit.forms.push(form_oid.clone());
                                                }
                                            }
                                        }
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