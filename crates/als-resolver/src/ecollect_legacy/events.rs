use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use entities::project::Visit;
use std::io::{Read, Seek};

/// Parse Events worksheet and populate context.visits.
pub fn parse_events(
    reader: impl Read + Seek,
    context: &mut crate::ecollect_legacy::LegacyParseContext,
) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader).map_err(|e: XlsxError| {
        crate::AlsParseError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    let range = workbook
        .worksheet_range("Events")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Events".to_string()))?;

    for row in range.rows().skip(1) {
        if row.len() < 3 {
            continue;
        }

        let oid = row[0].to_string();
        let sort_number: i32 = row[1].to_string().parse().unwrap_or(0);
        let name = row[2].to_string();

        if oid.is_empty() || oid == "OID" {
            continue;
        }

        let visit = Visit {
            code: oid.clone(),
            name,
            order: sort_number,
            forms: Vec::new(),
        };

        context.visits.insert(oid, visit);
    }

    Ok(())
}
