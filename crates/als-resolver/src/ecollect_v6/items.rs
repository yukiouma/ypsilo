use crate::ecollect_v6::context::{EcollectParseContext, ItemDef};
use calamine::{Reader, Xlsx, XlsxError, open_workbook_from_rs};
use std::io::{Read, Seek};

/// Parse Items worksheet and populate context.item_definitions.
/// Columns: ItemOID(0), SASFieldName(1), ItemName(2), ControlType(4),
/// DataFormat(7), CodeListOID(8), UnitGroupOID(11).
pub fn parse_items(
    reader: impl Read + Seek,
    context: &mut EcollectParseContext,
) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader).map_err(|e: XlsxError| {
        crate::AlsParseError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    let range = workbook
        .worksheet_range("Items")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Items".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 12 {
            continue;
        }

        let oid = row[0].to_string();
        if oid.is_empty() || oid == "ItemOID" {
            continue;
        }

        let code_list_raw = row[7].to_string();
        let unit_group_raw = row[11].to_string();

        let item_def = ItemDef {
            oid: oid.clone(),
            item_name: row[2].to_string(),
            sas_field_name: row[1].to_string(),
            control_type: row[4].to_string(),
            data_format: row[5].to_string(),
            code_list_oid: if code_list_raw.is_empty() {
                None
            } else {
                Some(EcollectParseContext::split_oid(&code_list_raw).to_string())
            },
            unit_group_oid: if unit_group_raw.is_empty() {
                None
            } else {
                Some(EcollectParseContext::split_oid(&unit_group_raw).to_string())
            },
        };

        context.item_definitions.insert(oid, item_def);
    }

    Ok(())
}
