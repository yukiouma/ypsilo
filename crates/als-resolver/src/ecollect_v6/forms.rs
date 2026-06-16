use calamine::{open_workbook, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::CRFForm;
use std::path::Path;

/// Parse Forms worksheet and populate context.forms.
/// Create CRFForm { name: FormOID, description: FormName, order: Ordinal, ... }.
pub fn parse_forms(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("Forms")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Forms".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 4 { continue; }

        let form_oid = row[0].to_string();
        let ordinal = row[1].to_string().parse::<i32>().unwrap_or(0);
        let form_name = row[3].to_string(); // FormName is column index 3 (0-based)

        if form_oid.is_empty() || form_oid == "FormOID" {
            continue;
        }

        let form = CRFForm {
            name: form_oid.clone(),
            description: form_name,
            order: ordinal,
            items: Vec::new(),
            domains: Vec::new(),
            annotations: Vec::new(),
        };

        context.forms.insert(form_oid, form);
    }

    Ok(())
}
