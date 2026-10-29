use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::sheet::SheetRows;
use quick_xml::Reader;
use std::collections::HashMap;

/// Parse the study matrix to populate Visit.forms.
///
/// A matrix sheet has a header row naming the visits it covers and one row
/// per form; an "X" in a visit column means the form is part of that visit.
pub fn parse_matrix_master<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    // Visit code of every column of the matrix, keyed by column index.
    let mut visit_codes: HashMap<usize, String> = HashMap::new();
    let mut is_header = true;

    for row in SheetRows::new(reader) {
        let row = row?;
        if is_header {
            is_header = false;
            for (column, value) in row.iter().enumerate() {
                // The first two columns hold the matrix name and "Subject",
                // every column after them is a visit.
                if column >= 2 && !value.is_empty() {
                    visit_codes.insert(column, value.clone());
                }
            }
            continue;
        }

        let form_oid = row.first().cloned().unwrap_or_default();
        if form_oid.is_empty() {
            continue;
        }

        for (column, value) in row.iter().enumerate() {
            if column < 2 || value != "X" {
                continue;
            }
            let Some(visit_code) = visit_codes.get(&column) else {
                continue;
            };
            if let Some(visit) = context.visits.iter_mut().find(|v| v.code == *visit_code)
                && !visit.forms.contains(&form_oid)
            {
                visit.forms.push(form_oid.clone());
            }
        }
    }

    Ok(())
}

/// Find the worksheet holding the study wide matrix.
///
/// Rave exports one worksheet per matrix; the study wide one is called
/// "MASTER" (or "ALL" in some exports) and is the matrix that binds forms to
/// visits. Returns `None` when the workbook has no such matrix.
pub fn find_study_matrix_sheet<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<Option<String>, AlsParseError> {
    let mut master: Option<String> = None;
    let mut all: Option<String> = None;
    // Position of the matrix in the Matrices worksheet, starting at 1 for the
    // first matrix. The worksheets are named after it: "Matrix1#ALL".
    let mut ordinal = 0usize;
    let mut is_header = true;

    for row in SheetRows::new(reader) {
        let row = row?;
        if is_header {
            is_header = false;
            continue;
        }
        ordinal += 1;

        // The second column of the Matrices worksheet holds the matrix OID,
        // which is also the suffix of the worksheet name.
        let Some(oid) = row.get(1).filter(|oid| !oid.is_empty()) else {
            continue;
        };
        if oid.eq_ignore_ascii_case("MASTER") {
            master = Some(format!("Matrix{ordinal}#{oid}"));
        } else if oid.eq_ignore_ascii_case("ALL") && all.is_none() {
            all = Some(format!("Matrix{ordinal}#{oid}"));
        }
    }

    Ok(master.or(all))
}
