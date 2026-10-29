use crate::error::AlsParseError;
use crate::rave::context::{FormRow, ParseContext};
use crate::rave::sheet::SheetRows;
use entities::project::CRFForm;
use quick_xml::Reader;

/// Parse the Forms worksheet (stop at worksheet boundary).
pub fn parse_forms<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    let mut is_header = true;

    for row in SheetRows::new(reader) {
        let row = row?;
        if is_header {
            is_header = false;
            continue;
        }
        if row.len() < 14 {
            continue;
        }

        let oid = row[0].clone();
        if oid.is_empty() || oid == "OID" {
            continue;
        }
        let ordinal = row[1].parse::<i32>().unwrap_or(0);
        let draft_form_name = row[2].clone();

        context.form_rows.push(FormRow {
            oid: oid.clone(),
            ordinal,
            draft_form_name: draft_form_name.clone(),
            // Column 13 of the sheet is LinkFolderOID.
            link_folder_oid: row.get(13).cloned(),
        });

        context.forms.insert(
            oid.clone(),
            CRFForm {
                name: oid,
                description: draft_form_name,
                order: ordinal,
                items: Vec::new(),
                domains: Vec::new(),
                annotations: Vec::new(),
            },
        );
    }

    Ok(())
}
