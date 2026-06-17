use als_resolver::parse_ecollect_v6_als;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[test]
fn test_parse_ecollect_v6_als_basic() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping integration test - .mock_data/als/ecollect_v6.xlsx not found");
        return;
    }

    let result = parse_ecollect_v6_als(path);
    assert!(
        result.is_ok(),
        "parse_ecollect_v6_als should succeed, got: {:?}",
        result.err()
    );

    let project = result.unwrap();
    assert!(
        !project.forms.is_empty(),
        "Project should have at least one form"
    );
    assert!(
        !project.visit.is_empty(),
        "Project should have at least one visit"
    );

    let first_form = &project.forms[0];
    assert!(!first_form.name.is_empty(), "Form name should not be empty");
    assert!(
        !first_form.description.is_empty(),
        "Form description should not be empty"
    );

    println!(
        "Parsed {} forms and {} visits",
        project.forms.len(),
        project.visit.len()
    );
}

#[test]
fn test_parse_ecollect_v6_als_forms_have_items() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    let forms_with_items = project.forms.iter().filter(|f| !f.items.is_empty()).count();
    assert!(forms_with_items > 0, "At least one form should have items");

    for form in &project.forms {
        for item in &form.items {
            assert!(!item.name.is_empty(), "Item name should not be empty");
            assert!(!item.label.is_empty(), "Item label should not be empty");
        }
    }
}

#[test]
fn test_parse_ecollect_v6_als_visit_form_bindings() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    let visits_with_forms = project.visit.iter().filter(|v| !v.forms.is_empty()).count();
    assert!(
        visits_with_forms > 0,
        "At least one visit should have form bindings"
    );

    for visit in &project.visit {
        assert!(!visit.code.is_empty(), "Visit code should not be empty");
        assert!(!visit.name.is_empty(), "Visit name should not be empty");
    }
}

#[test]
fn test_parse_ecollect_v6_als_control_types() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    let control_types: HashSet<_> = project
        .forms
        .iter()
        .flat_map(|f| f.items.iter().map(|i| &i.control_type))
        .collect();

    assert!(
        !control_types.is_empty(),
        "Should have at least one control type"
    );

    use entities::project::ControlType;
    for ct in &control_types {
        match ct {
            ControlType::TEXT
            | ControlType::SELECTION
            | ControlType::CHECKBOX
            | ControlType::DATETIME => {}
        }
    }
}

#[test]
fn test_parse_ecollect_v6_als_not_variable_for_tags() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    let items_with_not_variable = project
        .forms
        .iter()
        .flat_map(|f| f.items.iter().filter(|i| i.not_variable == Some(true)))
        .collect::<Vec<_>>();

    for item in &items_with_not_variable {
        assert!(
            matches!(item.control_type, entities::project::ControlType::TEXT),
            "Items with not_variable=true should have TEXT control type"
        );
    }
}

#[test]
fn test_parse_ecollect_v6_als_item_options() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    // Verify that any items with options have valid non-empty option displays
    for form in &project.forms {
        for item in &form.items {
            if let Some(ref options) = item.item_option {
                assert!(!options.is_empty(), "Options list should not be empty");
                for opt in options {
                    assert!(
                        !opt.option_display.is_empty(),
                        "Option display value should not be empty"
                    );
                }
            }
        }
    }
}

#[test]
fn test_parse_ecollect_v6_als_form_count() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    // ecollect_v6.md says Forms has ~40 rows
    assert!(
        project.forms.len() >= 38 && project.forms.len() <= 42,
        "Should have 38-42 forms, got {}",
        project.forms.len()
    );
}

#[test]
fn test_parse_ecollect_v6_als_visit_count() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    // ecollect_v6.md says Plan* sheets have visit columns
    assert!(
        project.visit.len() >= 15 && project.visit.len() <= 25,
        "Should have between 15-25 visits, got {}",
        project.visit.len()
    );
}

#[test]
fn test_parse_ecollect_v6_als_debug_items() {
    let path = Path::new("../../.mock_data/als/ecollect_v6.xlsx");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_ecollect_v6_als(path).unwrap();

    let mut with_options = 0;
    let mut without_options = 0;
    let mut total_items = 0;
    let mut control_types: HashMap<String, usize> = HashMap::new();
    let mut sample_items: Vec<(String, String, String)> = Vec::new();

    for form in &project.forms {
        for item in &form.items {
            total_items += 1;
            *control_types
                .entry(format!("{:?}", item.control_type))
                .or_insert(0) += 1;

            if item.item_option.is_some() {
                with_options += 1;
            } else {
                without_options += 1;
            }

            if sample_items.len() < 5 && !item.label.is_empty() {
                sample_items.push((
                    item.name.clone(),
                    item.label.clone(),
                    format!("{:?}", item.control_type),
                ));
            }
        }
    }

    println!("Total items: {}", total_items);
    println!("Items with options: {}", with_options);
    println!("Items without options: {}", without_options);
    println!("Control types: {:?}", control_types);
    println!("Sample items: {:?}", sample_items);
}
