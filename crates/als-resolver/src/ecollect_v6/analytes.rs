use calamine::{open_workbook, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use std::path::Path;

/// Parse AnalytesInTheStudy worksheet and populate context.analytes.
/// Build AnalyteCode → AnalyteName lookup for Lab Test / Lab Result options.
pub fn parse_analytes(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("AnalytesInTheStudy")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("AnalytesInTheStudy".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 2 { continue; }

        let analyte_code = row[0].to_string();
        let analyte_name = row[1].to_string();

        if analyte_code.is_empty() || analyte_code == "AnalytesCode" {
            continue;
        }

        context.analytes.insert(analyte_code, analyte_name);
    }

    Ok(())
}
