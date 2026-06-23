use als_resolver::parse_rave_als;
use std::io::Cursor;
use std::path::Path;

fn read_rave_bytes() -> Option<Vec<u8>> {
    let path = Path::new("../../.mock_data/als/rave.xml");
    if !path.exists() {
        None
    } else {
        Some(std::fs::read(path).unwrap())
    }
}

#[test]
fn test_parse_rave_als_integration() {
    let Some(bytes) = read_rave_bytes() else {
        eprintln!("Skipping integration test - .mock_data/als/rave.xml not found");
        return;
    };

    let result = parse_rave_als(Cursor::new(bytes));
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
    let Some(bytes) = read_rave_bytes() else {
        eprintln!("Skipping - .mock_data/als/rave.xml not found");
        return;
    };

    let project = parse_rave_als(Cursor::new(bytes)).unwrap();

    // Verify form-item relationship
    for form in &project.forms {
        for item in &form.items {
            assert!(!item.name.is_empty(), "Item name should not be empty");
            assert!(!item.label.is_empty(), "Item label should not be empty");
        }
    }
}

#[test]
fn test_parse_rave_als_from_reader() {
    let Some(bytes) = read_rave_bytes() else {
        eprintln!("Skipping - .mock_data/als/rave.xml not found");
        return;
    };

    let cursor = Cursor::new(bytes);

    let result = parse_rave_als(cursor);
    assert!(result.is_ok(), "parse_rave_als should succeed");

    let project = result.unwrap();
    assert!(!project.forms.is_empty(), "Project should have forms");
    assert!(!project.visit.is_empty(), "Project should have visits");
}

#[test]
fn test_parse_rave_als_consistency() {
    let Some(bytes) = read_rave_bytes() else {
        eprintln!("Skipping - .mock_data/als/rave.xml not found");
        return;
    };

    let cursor = Cursor::new(bytes);
    let project = parse_rave_als(cursor).unwrap();

    assert!(!project.forms.is_empty(), "Forms should not be empty");
    assert!(!project.visit.is_empty(), "Visits should not be empty");

    // Verify form-item relationship
    for form in &project.forms {
        for item in &form.items {
            assert!(!item.name.is_empty(), "Item name should not be empty");
            assert!(!item.label.is_empty(), "Item label should not be empty");
        }
    }
}
