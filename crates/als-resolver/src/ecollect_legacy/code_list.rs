use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use entities::project::ItemOption;
use std::io::{Read, Seek};

/// Parse CodeListItems worksheet and populate context.code_list_options.
pub fn parse_code_list_items(
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
        .worksheet_range("CodeListItems")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("CodeListItems".to_string()))?;

    for row in range.rows().skip(1) {
        if row.len() < 3 {
            continue;
        }

        let code_list_oid = row[0].to_string();
        let display_value = row[2].to_string();

        if code_list_oid.is_empty() || code_list_oid == "CodeListOID" {
            continue;
        }

        let option = ItemOption {
            option_display: display_value,
            annotations: Vec::new(),
        };

        context
            .code_list_options
            .entry(code_list_oid)
            .or_default()
            .push(option);
    }

    Ok(())
}
