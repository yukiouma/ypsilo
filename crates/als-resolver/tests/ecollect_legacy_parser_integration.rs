use als_resolver::parse_ecollect_legacy_als;
use std::collections::HashSet;
use std::path::Path;

fn get_legacy_path() -> &'static Path {
    Path::new("../../.mock_data/als/ecollect_legacy.xlsx")
}

#[test]
fn test_parse_ecollect_legacy_als_basic() {
    let path = get_legacy_path();
    if !path.exists() {
        eprintln!("Skipping - .mock_data/als/ecollect_legacy.xlsx not found");
        return;
    }

    let result = parse_ecollect_legacy_als(&path);
    assert!(result.is_ok(), "parse_ecollect_legacy_als should succeed: {:?}", result.err());
    let project = result.unwrap();
    assert!(!project.forms.is_empty(), "Project should have forms");
    assert!(!project.visit.is_empty(), "Project should have visits");
}

#[test]
fn test_parse_ecollect_legacy_als_forms_have_items() {
    let path = get_legacy_path();
    if !path.exists() { return; }

    let project = parse_ecollect_legacy_als(&path).unwrap();
    let forms_with_items = project.forms.iter().filter(|f| !f.items.is_empty()).count();
    assert!(forms_with_items > 0, "At least one form should have items");
}

#[test]
fn test_parse_ecollect_legacy_als_visit_form_bindings() {
    let path = get_legacy_path();
    if !path.exists() { return; }

    let project = parse_ecollect_legacy_als(&path).unwrap();
    let visits_with_forms = project.visit.iter().filter(|v| !v.forms.is_empty()).count();
    assert!(visits_with_forms > 0, "At least one visit should have forms");
}

#[test]
fn test_parse_ecollect_legacy_als_control_types() {
    let path = get_legacy_path();
    if !path.exists() { return; }

    let project = parse_ecollect_legacy_als(&path).unwrap();
    let control_types: HashSet<_> = project
        .forms
        .iter()
        .flat_map(|f| f.items.iter().map(|i| &i.control_type))
        .collect();
    assert!(!control_types.is_empty(), "Should have control types");

    use entities::project::ControlType;
    for ct in &control_types {
        match ct {
            ControlType::TEXT | ControlType::SELECTION | ControlType::CHECKBOX | ControlType::DATETIME => {}
        }
    }
}

#[test]
fn test_parse_ecollect_legacy_als_item_options() {
    let path = get_legacy_path();
    if !path.exists() { return; }

    let project = parse_ecollect_legacy_als(&path).unwrap();
    for form in &project.forms {
        for item in &form.items {
            if let Some(ref options) = item.item_option {
                assert!(!options.is_empty(), "Options list should not be empty");
                for opt in options {
                    assert!(!opt.option_display.is_empty(), "Option display should not be empty");
                }
            }
        }
    }
}

#[test]
fn test_parse_ecollect_legacy_als_not_variable() {
    let path = get_legacy_path();
    if !path.exists() { return; }

    let project = parse_ecollect_legacy_als(&path).unwrap();
    let items_with_not_var = project
        .forms
        .iter()
        .flat_map(|f| f.items.iter().filter(|i| i.not_variable == Some(true)))
        .collect::<Vec<_>>();

    for item in &items_with_not_var {
        // not_variable=true items can be TEXT, SELECTION, or CHECKBOX based on actual data
        assert!(
            matches!(item.control_type, entities::project::ControlType::TEXT | entities::project::ControlType::SELECTION | entities::project::ControlType::CHECKBOX),
            "Items with not_variable=true should have TEXT, SELECTION, or CHECKBOX control type, got {:?}", item.control_type
        );
    }
}
