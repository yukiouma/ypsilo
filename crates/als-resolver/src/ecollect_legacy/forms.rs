use calamine::{open_workbook, Reader, Xlsx, XlsxError};
use entities::project::CRFForm;
use std::path::Path;

/// Parse Forms worksheet and populate context.forms.
pub fn parse_forms(
    path: &Path,
    context: &mut crate::ecollect_legacy::LegacyParseContext,
) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e: XlsxError| {
        crate::AlsParseError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    let range = workbook
        .worksheet_range("Forms")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Forms".to_string()))?;

    for row in range.rows().skip(1) {
        if row.len() < 3 {
            continue;
        }

        let oid = row[0].to_string();
        let name = row[2].to_string();

        if oid.is_empty() || oid == "OID" {
            continue;
        }

        let form = CRFForm {
            name: oid.clone(),
            description: name,
            order: 0,
            items: Vec::new(),
            domains: Vec::new(),
            annotations: Vec::new(),
        };

        context.forms.insert(oid, form);
    }

    Ok(())
}
