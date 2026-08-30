use calamine::{Reader, Xlsx, XlsxError, open_workbook_from_rs};
use entities::project::{CRFItem, ControlType, ItemOption};
use std::io::{Read, Seek};

/// Parse GroupItems worksheet and populate context.forms with CRFItem entries.
pub fn parse_group_items(
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
        .worksheet_range("GroupItems")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("GroupItems".to_string()))?;

    for row in range.rows().skip(1) {
        if row.len() < 29 {
            continue;
        }

        let form_oid = row[0].to_string();
        let item_oid = row[3].to_string();
        let display_mode = row[15].to_string();
        let data_format = row[16].to_string();
        let item_name = row[18].to_string();
        let code_list_oid = row[20].to_string();
        let check_field_required_str = row[27].to_string();
        let default_value = row[26].to_string();

        if form_oid.is_empty() || form_oid == "FormOID" {
            continue;
        }
        if item_oid.is_empty() {
            continue;
        }

        // Filter out items where CheckFieldRequired is "Disable" AND AllowedEntry is "Y".
        // Such items are non-variable fields the study explicitly disables, so they
        // should not appear in the parsed CRF.
        if display_mode == "Hidden" {
            continue;
        }

        // Determine item options based on DisplayMode
        let item_option = if display_mode == "AnalytesOption" {
            // DefaultValue contains pipe-separated analyte codes; split and look up names
            let options: Vec<ItemOption> = default_value
                .split('|')
                .filter_map(|code| {
                    let code = code.trim();
                    if code.is_empty() {
                        return None;
                    }
                    context.analytes.get(code).map(|name| ItemOption {
                        option_display: name.clone(),
                        annotations: Vec::new(),
                    })
                })
                .collect();
            if options.is_empty() {
                None
            } else {
                Some(options)
            }
        } else if !code_list_oid.is_empty() {
            context.code_list_options.get(&code_list_oid).cloned()
        } else {
            None
        };

        let control_type = match display_mode.as_str() {
            "RadioButton" => ControlType::SELECTION,
            "CheckBox" => ControlType::CHECKBOX,
            "DropDownList" => ControlType::SELECTION,
            "TextField" => ControlType::TEXT,
            "Date" => ControlType::DATETIME,
            "File" => ControlType::TEXT,
            "AnalytesOption" => ControlType::SELECTION,
            _ => ControlType::TEXT,
        };

        // CheckFieldRequired = "Disable" means not_variable = true
        let not_variable = Some(check_field_required_str.to_lowercase() == "disable");

        let item = CRFItem {
            name: item_oid,
            label: item_name,
            item_option,
            annotations: Vec::new(),
            format: data_format,
            control_type,
            item_unit: None,
            not_variable,
        };

        if let Some(form) = context.forms.get_mut(&form_oid) {
            form.items.push(item);
        }
    }

    Ok(())
}
