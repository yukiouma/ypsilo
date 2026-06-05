use als_resolver::parse_rave_als;
use std::path::Path;

#[test]
fn test_parse_rave_als_integration() {
    let path = Path::new("../../.mock_data/als/rave.xml");
    if !path.exists() {
        eprintln!("Skipping integration test - .mock_data/als/rave.xml not found");
        return;
    }

    let result = parse_rave_als(path);
    assert!(result.is_ok(), "parse_rave_als should succeed");

    let project = result.unwrap();
    assert!(
        !project.forms.is_empty(),
        "Project should have at least one form"
    );
    assert!(
        !project.visit.is_empty(),
        "Project should have at least one visit"
    );

    // Check first form structure
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
fn test_parse_rave_als_with_file() {
    let path = Path::new("../../.mock_data/als/rave.xml");
    if !path.exists() {
        eprintln!("Skipping - mock data not found");
        return;
    }

    let project = parse_rave_als(path).unwrap();

    // Verify form-item relationship
    for form in &project.forms {
        for item in &form.items {
            assert!(!item.name.is_empty(), "Item name should not be empty");
            assert!(!item.label.is_empty(), "Item label should not be empty");
        }
    }
}
