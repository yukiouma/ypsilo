use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::sheet::SheetRows;
use entities::project::{CRFItem, ControlType};
use quick_xml::Reader;

/// Parse the Fields worksheet and populate form items (stop at worksheet boundary).
pub fn parse_fields<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    context: &mut ParseContext,
) -> Result<(), AlsParseError> {
    for row in SheetRows::new(reader) {
        let row = row?;
        if row.len() < 37 || row[0] == "FormOID" {
            // Not a field row — either the header or a row without the columns
            // the parser reads.
            continue;
        }
        // Skip fields whose ViewRestrictions column (index 41, column AP) is
        // populated — those fields are restricted to specific roles and should
        // not appear in the CRF.
        if row.get(41).is_some_and(|value| !value.is_empty()) {
            continue;
        }

        let form_oid = row[0].clone();
        let field_oid = row[1].clone();
        let _ordinal = row[2].parse::<i32>().unwrap_or(0);
        let draft_field_name = row[4].clone();
        let variable_oid = row[6].clone();
        let data_format = row[7].clone();
        let data_dictionary_name = row[8].clone();
        let control_type_str = row[11].clone();
        let pre_text = row[14].clone();
        let fixed_unit = row[15].clone();

        // Get options from DataDictionary if present
        let item_option = if data_dictionary_name.is_empty() {
            None
        } else {
            let options = context.get_options(&data_dictionary_name);
            if options.is_empty() {
                None
            } else {
                Some(options)
            }
        };

        // Map control type string to ControlType enum.
        // The keys here are the literal strings produced by RAVE in the
        // Fields worksheet; anything unrecognized falls back to TEXT.
        let control_type = match control_type_str.as_str() {
            "DateTime" => ControlType::DATETIME,
            "CheckBox" => ControlType::CHECKBOX,
            "DropDownList" => ControlType::SELECTION,
            "RadioButton" => ControlType::SELECTION,
            "RadioButton (Vertical)" => ControlType::SELECTION,
            _ => ControlType::TEXT,
        };

        // Use PreText as label if available, otherwise DraftFieldName
        let label = if !pre_text.is_empty() {
            pre_text
        } else if !draft_field_name.is_empty() {
            draft_field_name
        } else {
            "".into()
        };

        let item_unit = if fixed_unit.is_empty() {
            None
        } else {
            Some(entities::project::ItemUnit {
                value: fixed_unit,
                annotations: Vec::new(),
            })
        };

        let item = CRFItem {
            name: field_oid,
            label,
            item_option,
            annotations: Vec::new(),
            format: data_format,
            control_type,
            item_unit,
            not_variable: if variable_oid.is_empty() {
                Some(true)
            } else {
                None
            },
        };

        // Add item to the corresponding form
        if let Some(form) = context.forms.get_mut(&form_oid) {
            form.items.push(item);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities::project::CRFForm;

    /// Build a Fields row of at least 37 cells (the minimum the parser accepts)
    /// where every cell is empty except for the columns the parser reads.
    fn build_row_xml(pre_text: &str, form_oid: &str, field_oid: &str) -> String {
        build_row_xml_with_view_restrictions(pre_text, form_oid, field_oid, None)
    }

    /// Same as `build_row_xml` but lets the caller set the ViewRestrictions
    /// column (index 41). The row is padded to 42 cells when a value is
    /// supplied so the filter actually has something to inspect.
    fn build_row_xml_with_view_restrictions(
        pre_text: &str,
        form_oid: &str,
        field_oid: &str,
        view_restrictions: Option<&str>,
    ) -> String {
        let target_len = if view_restrictions.is_some() { 42 } else { 37 };
        let mut cells: Vec<String> = (0..target_len)
            .map(|_| String::from("<Cell><Data></Data></Cell>"))
            .collect();
        cells[0] = format!("<Cell><Data>{form_oid}</Data></Cell>");
        cells[1] = format!("<Cell><Data>{field_oid}</Data></Cell>");
        cells[2] = String::from("<Cell><Data>1</Data></Cell>");
        cells[4] = format!("<Cell><Data>{field_oid}</Data></Cell>");
        cells[11] = String::from("<Cell><Data>Text</Data></Cell>");
        // Escape the pre_text for safe XML embedding.
        let escaped = pre_text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        cells[14] = format!("<Cell><Data>{escaped}</Data></Cell>");
        if let Some(vr) = view_restrictions {
            let escaped_vr = vr
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            cells[41] = format!("<Cell><Data>{escaped_vr}</Data></Cell>");
        }
        format!("<Row>{}</Row>", cells.join(""))
    }

    fn make_context(form_oid: &str) -> ParseContext {
        let mut context = ParseContext::new();
        context.forms.insert(
            form_oid.to_string(),
            CRFForm {
                name: form_oid.to_string(),
                description: form_oid.to_string(),
                order: 0,
                items: Vec::new(),
                domains: Vec::new(),
                annotations: Vec::new(),
            },
        );
        context
    }

    fn parse(xml: &str, context: &mut ParseContext) {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().expand_empty_elements = true;
        parse_fields(&mut reader, context).unwrap();
    }

    #[test]
    fn cell_with_entity_reference_is_not_split() {
        // This is the exact scenario from the bug report: a PreText cell whose
        // raw value contains `&gt;` must be parsed as a single, intact string.
        let pre_text = "Infusion-related reactions (grade>= 3)";
        let xml = build_row_xml(pre_text, "SV", "SVFOO");
        let mut context = make_context("SV");

        parse(&xml, &mut context);

        let form = context.forms.get("SV").expect("form should exist");
        assert_eq!(form.items.len(), 1, "exactly one item should be parsed");
        assert_eq!(form.items[0].label, pre_text);
    }

    #[test]
    fn cell_with_numeric_character_reference_is_parsed() {
        // Rave stores multi line values (edit check code) using numeric
        // character references for the line breaks.
        let xml = r#"<Row>
            <Cell><Data>SV</Data></Cell>
            <Cell><Data>SVMULTI</Data></Cell>
            <Cell><Data>1</Data></Cell>
            <Cell ss:Index="5"><Data>SVMULTI</Data></Cell>
            <Cell ss:Index="6"><Data>TRUE</Data></Cell>
            <Cell ss:Index="7"><Data>SITE_ID</Data></Cell>
            <Cell ss:Index="8"><Data>$4</Data></Cell>
            <Cell ss:Index="12"><Data>Text</Data></Cell>
            <Cell ss:Index="15"><Data>first line&#13;&#10;second line</Data></Cell>
            <Cell ss:Index="37"><Data>unused</Data></Cell>
        </Row>"#;
        let mut context = make_context("SV");

        parse(xml, &mut context);

        let form = context.forms.get("SV").unwrap();
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].label, "first line\r\nsecond line");
    }

    #[test]
    fn cell_without_entity_reference_still_parses() {
        // Regression check: cells with plain text must still end up in the
        // correct slot after the cell-indexing change.
        let xml = build_row_xml("Visit Date", "SV", "SVDAT");
        let mut context = make_context("SV");

        parse(&xml, &mut context);

        let form = context.forms.get("SV").unwrap();
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].label, "Visit Date");
        assert_eq!(form.items[0].name, "SVDAT");
    }

    #[test]
    fn field_with_view_restrictions_is_filtered_out() {
        // Column AP (index 41) is "ViewRestrictions". When populated, the
        // field is restricted to specific roles and must not be added to the
        // form's items.
        let xml = build_row_xml_with_view_restrictions(
            "Restricted Field",
            "SV",
            "SVRESTRICTED",
            Some("Investigator"),
        );
        let mut context = make_context("SV");

        parse(&xml, &mut context);

        let form = context.forms.get("SV").unwrap();
        assert!(
            form.items.is_empty(),
            "field with ViewRestrictions must be filtered out, got {} item(s)",
            form.items.len()
        );
    }

    #[test]
    fn field_with_empty_view_restrictions_is_kept() {
        // A blank ViewRestrictions cell means the field is not restricted
        // and must be added normally.
        let xml = build_row_xml_with_view_restrictions("Open Field", "SV", "SVOPEN", Some(""));
        let mut context = make_context("SV");

        parse(&xml, &mut context);

        let form = context.forms.get("SV").unwrap();
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].label, "Open Field");
        assert_eq!(form.items[0].name, "SVOPEN");
    }

    #[test]
    fn cells_are_placed_by_their_ss_index() {
        // A cell carrying ss:Index is placed at that (1-based) column, the
        // other cells follow it.
        let xml = r#"<Row>
            <Cell><Data>SV</Data></Cell>
            <Cell><Data>SVDAT</Data></Cell>
            <Cell><Data>1</Data></Cell>
            <Cell ss:Index="5"><Data>SVDAT</Data></Cell>
            <Cell ss:Index="6"><Data>TRUE</Data></Cell>
            <Cell ss:Index="7"><Data>SITE_ID</Data></Cell>
            <Cell ss:Index="8"><Data>$4</Data></Cell>
            <Cell ss:Index="12"><Data>Text</Data></Cell>
            <Cell ss:Index="15"><Data>Visit Date</Data></Cell>
            <Cell ss:Index="37"><Data>unused</Data></Cell>
        </Row>"#;
        let mut context = make_context("SV");

        parse(xml, &mut context);

        let form = context.forms.get("SV").unwrap();
        assert_eq!(form.items.len(), 1);
        assert_eq!(form.items[0].name, "SVDAT");
        assert_eq!(form.items[0].label, "Visit Date");
        assert_eq!(form.items[0].format, "$4");
    }
}
