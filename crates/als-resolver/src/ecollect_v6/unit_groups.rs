use calamine::{open_workbook, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use std::path::Path;

/// Parse Units worksheet into context.unit_groups.
/// Build UnitGroupOID → Vec<UnitName> lookup for item_unit resolution.
pub fn parse_unit_groups(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    // Parse Units worksheet (UnitGroupOID, UnitOID, UnitName, ...)
    let units_range = workbook.worksheet_range("Units")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Units".to_string()))?;

    // First row is header, skip it
    for row in units_range.rows().skip(1) {
        if row.len() < 3 { continue; }

        let unit_group_oid = row[0].to_string();
        let unit_name = row[2].to_string();

        if unit_group_oid.is_empty() || unit_group_oid == "UnitGroupOID" {
            continue;
        }

        context.unit_groups
            .entry(unit_group_oid)
            .or_default()
            .push(unit_name);
    }

    Ok(())
}
