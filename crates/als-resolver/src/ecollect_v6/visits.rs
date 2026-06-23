use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::Visit;
use std::io::{Read, Seek};

/// Parse Plan* sheets and build Visit structs.
/// Visit code = column header (columns 1+), name from formset_names lookup.
/// Build visit_form_bindings from non-empty cells in Plan* sheets.
pub fn parse_visits(reader: impl Read + Seek, context: &mut EcollectParseContext) -> Result<Vec<Visit>, crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader).map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let plan_sheets = ["PlanSCR", "PlanCYCLE", "PlanEARLY", "PlanCOM", "PlanDSEOS", "PlanUNS"];

    // First pass: extract column headers from first Plan* sheet to get visit codes
    let first_sheet = plan_sheets.first().unwrap();
    let range = workbook.worksheet_range(first_sheet)
        .map_err(|_| crate::AlsParseError::WorksheetNotFound(first_sheet.to_string()))?;

    let mut visit_codes: Vec<String> = Vec::new();
    if let Some(header_row) = range.rows().next() {
        // Column 0 = "Form\\Visit", columns 1+ = visit codes
        for (i, cell) in header_row.iter().enumerate() {
            if i == 0 { continue; } // Skip "Form\\Visit"
            let code = cell.to_string();
            if !code.is_empty() {
                visit_codes.push(code);
            }
        }
    }

    // Second pass: process all Plan* sheets to build visit_form_bindings
    for sheet_name in &plan_sheets {
        let Ok(sheet_range) = workbook.worksheet_range(sheet_name) else {
            continue;
        };

        for (row_idx, row) in sheet_range.rows().enumerate() {
            if row_idx == 0 { continue; } // Skip header row
            if row.is_empty() { continue; }

            let form_oid = row[0].to_string();
            if form_oid.is_empty() { continue; }

            // Check columns 1+ for non-empty cells
            for (col_idx, cell) in row.iter().enumerate().skip(1) {
                let cell_str = cell.to_string();
                if !cell_str.is_empty() && col_idx - 1 < visit_codes.len() {
                    let visit_code = &visit_codes[col_idx - 1];
                    context.visit_form_bindings
                        .entry(visit_code.clone())
                        .or_default()
                        .push(form_oid.clone());
                }
            }
        }
    }

    // Build Visit structs
    let mut visits: Vec<Visit> = Vec::new();
    for (order, code) in visit_codes.iter().enumerate() {
        let name = context.formset_names.get(code).cloned().unwrap_or_else(|| code.clone());
        let forms = context.visit_form_bindings.get(code).cloned().unwrap_or_default();

        // Deduplicate forms
        let mut unique_forms: Vec<String> = Vec::new();
        for f in forms {
            if !unique_forms.contains(&f) {
                unique_forms.push(f);
            }
        }

        visits.push(Visit {
            code: code.clone(),
            name,
            order: order as i32,
            forms: unique_forms,
        });
    }

    Ok(visits)
}
