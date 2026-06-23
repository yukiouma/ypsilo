use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use std::io::{Read, Seek};

/// Parse AnalytesInTheStudy worksheet and populate context.analytes.
/// Build AnalyteCode → AnalyteName lookup for Lab Test / Lab Result options.
pub fn parse_analytes(reader: impl Read + Seek, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader).map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

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
