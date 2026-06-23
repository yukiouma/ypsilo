use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use std::io::{Read, Seek};

/// Parse EventForm worksheet and build event-form linkages.
pub fn parse_event_form(
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
        .worksheet_range("EventForm")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("EventForm".to_string()))?;

    for row in range.rows().skip(1) {
        if row.len() < 3 {
            continue;
        }

        let event_oid = row[0].to_string();
        let form_oid = row[2].to_string();

        if event_oid.is_empty() || event_oid == "EventOID" {
            continue;
        }
        if form_oid.is_empty() {
            continue;
        }

        context
            .event_form_bindings
            .entry(event_oid)
            .or_default()
            .push(form_oid);
    }

    Ok(())
}
