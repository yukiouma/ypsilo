use calamine::{open_workbook_from_rs, Reader, Xlsx, Data};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::{CRFItem, ControlType, ItemOption, ItemUnit};
use std::io::{Read, Seek};

/// Parse FormItem worksheet and populate form.items with CRFItems.
/// For each row, look up ItemOID in item_definitions, create CRFItem with
/// ControlType mapping, CodeList/Lab Test options, unit resolution.
pub fn parse_form_item(reader: impl Read + Seek, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader).map_err(|e: calamine::XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("FormItem")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("FormItem".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        let row: &[Data] = row;
        if row.len() < 3 { continue; }

        let form_oid = row[0].to_string();
        let item_oid = row[2].to_string();

        if form_oid.is_empty() || form_oid == "FormOID" || item_oid.is_empty() || item_oid == "ItemOID" {
            continue;
        }

        // Filter out bindings where ItemViewRestriction (column AG, index 32) is "isAll".
        if row.get(32).map(|c| c.to_string()).as_deref() == Some("isAll") {
            continue;
        }

        // Look up item definition
        let Some(item_def) = context.item_definitions.get(&item_oid) else {
            continue;
        };

        // Resolve control_type
        let (control_type, not_variable) = map_control_type(&item_def.control_type);

        // Resolve item_option from FormItem's own DefaultValue (col P) and
        // CodeListOID (col AU). The Items.CodeListOID is empty for Lab Test
        // items, so we must read these columns directly from FormItem.
        let default_value = row.get(15).map(|c: &Data| c.to_string());
        let code_list_oid_raw = row.get(46).map(|c: &Data| c.to_string());
        let item_option = resolve_item_option(
            &item_def.control_type,
            default_value.as_deref(),
            code_list_oid_raw.as_deref(),
            context,
        );

        // Resolve item_unit
        let item_unit = resolve_item_unit(&item_def.unit_group_oid, context);

        // Label from FormItem.ItemName (field 41, index 41) or Items.ItemName
        let label = if row.len() > 41 && !row[41].to_string().is_empty() {
            row[41].to_string()
        } else {
            item_def.item_name.clone()
        };

        let item = CRFItem {
            name: item_oid.clone(),
            label,
            item_option,
            annotations: Vec::new(),
            format: item_def.data_format.clone(),
            control_type,
            item_unit,
            not_variable,
        };

        // Add item to form
        if let Some(form) = context.forms.get_mut(&form_oid) {
            form.items.push(item);
        }
    }

    Ok(())
}

/// Map ecollect ControlType string to CRFItem ControlType enum and not_variable.
fn map_control_type(ct: &str) -> (ControlType, Option<bool>) {
    match ct {
        "Textbox" => (ControlType::TEXT, None),
        "Drop-down List" => (ControlType::SELECTION, None),
        "Radio(horizontal)" => (ControlType::SELECTION, None),
        "Radio(vertical)" => (ControlType::SELECTION, None),
        "Check" => (ControlType::CHECKBOX, None),
        "Tags" => (ControlType::TEXT, Some(true)),
        "Lab Test" => (ControlType::SELECTION, None),
        "Lab Result" => (ControlType::SELECTION, None),
        "Calendar" => (ControlType::TEXT, None),
        "Dynamic Options" => (ControlType::TEXT, None),
        _ => (ControlType::TEXT, None),
    }
}

/// Resolve item_option from FormItem's DefaultValue (Lab Test / Lab Result)
/// or CodeListOID (other selection items). The Items.CodeListOID column is
/// empty for Lab Test items, so the FormItem row is the source of truth.
fn resolve_item_option(
    control_type: &str,
    default_value: Option<&str>,
    code_list_oid: Option<&str>,
    context: &EcollectParseContext,
) -> Option<Vec<ItemOption>> {
    match control_type {
        "Lab Test" | "Lab Result" => {
            // DefaultValue carries analyte codes separated by "|", e.g.
            // "TSH_A|T3FR_A|T4FR_A". Resolve each via AnalytesInTheStudy.
            let Some(dv) = default_value else { return None };
            let mut options = Vec::new();
            for code in dv.split('|') {
                let code = code.trim();
                if code.is_empty() {
                    continue;
                }
                if let Some(analyte_name) = context.analytes.get(code) {
                    options.push(ItemOption {
                        option_display: analyte_name.clone(),
                        annotations: Vec::new(),
                    });
                }
            }
            if options.is_empty() { None } else { Some(options) }
        }
        _ => {
            // CodeListOID may be compound like "YN=[1|是,2|否]"; take the part
            // before the first "=" as the code list key.
            let Some(raw) = code_list_oid else { return None };
            let oid = EcollectParseContext::split_oid(raw);
            context.code_list_options.get(oid).cloned()
        }
    }
}

/// Resolve item_unit from UnitGroupOID.
fn resolve_item_unit(
    unit_group_oid: &Option<String>,
    context: &EcollectParseContext,
) -> Option<ItemUnit> {
    if let Some(oid) = unit_group_oid {
        if let Some(units) = context.unit_groups.get(oid) {
            if let Some(first_unit) = units.first() {
                return Some(ItemUnit {
                    value: first_unit.clone(),
                    annotations: Vec::new(),
                });
            }
        }
    }
    None
}
