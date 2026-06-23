use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use std::io::{Read, Seek};

/// Parse FormSets worksheet and populate context.formset_names.
/// Build FormsetOID → FormsetName lookup for visit name resolution.
pub fn parse_form_sets(reader: impl Read + Seek, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader).map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("FormSets")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("FormSets".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 2 { continue; }

        let formset_oid = row[0].to_string();
        let formset_name = row[1].to_string();

        if formset_oid.is_empty() || formset_oid == "FormsetOID" {
            continue;
        }

        context.formset_names.insert(formset_oid, formset_name);
    }

    Ok(())
}
