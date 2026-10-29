use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::sheet::SheetRows;
use entities::project::Visit;
use quick_xml::Reader;

/// Parse the Folders worksheet to create Visit structs.
pub fn parse_folders<R: std::io::BufRead>(
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
        if row.len() < 3 {
            continue;
        }

        let oid = row[0].clone();
        if oid.is_empty() || oid == "OID" {
            continue;
        }
        let ordinal = row[1].parse::<i32>().unwrap_or(0);
        let folder_name = row[2].clone();

        context.visits.push(Visit {
            code: oid,
            name: folder_name,
            order: ordinal,
            forms: Vec::new(),
        });
    }

    Ok(())
}
