use als_resolver::parse_rave_als;
use std::path::{Path, PathBuf};

/// The mock Rave workbooks shipped with the repository.
fn mock_rave_files() -> Vec<PathBuf> {
    let directory = Path::new("../../.mock_data/als/rave");
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xml"))
        .collect();
    files.sort();
    files
}

/// The number of forms and visits each mock workbook holds.
fn expected_counts(file_name: &str) -> Option<(usize, usize)> {
    match file_name {
        "rave00.xml" => Some((70, 77)),
        "rave01.xml" => Some((71, 90)),
        "rave02.xml" => Some((61, 56)),
        "rave03.xml" => Some((63, 91)),
        _ => None,
    }
}

#[test]
fn test_parse_all_mock_rave_files() {
    let files = mock_rave_files();
    if files.is_empty() {
        eprintln!("Skipping - .mock_data/als/rave not found");
        return;
    }

    for path in files {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        let project = parse_rave_als(std::fs::File::open(&path).unwrap())
            .unwrap_or_else(|e| panic!("{file_name} should parse: {e}"));

        assert!(
            !project.forms.is_empty(),
            "{file_name}: project should have at least one form"
        );
        assert!(
            !project.visit.is_empty(),
            "{file_name}: project should have at least one visit"
        );

        // Check first form structure
        let first_form = &project.forms[0];
        assert!(
            !first_form.name.is_empty(),
            "{file_name}: form name should not be empty"
        );
        assert!(
            !first_form.description.is_empty(),
            "{file_name}: form description should not be empty"
        );

        // Verify form-item relationship
        for form in &project.forms {
            for item in &form.items {
                assert!(
                    !item.name.is_empty(),
                    "{file_name}: item name should not be empty"
                );
                assert!(
                    !item.label.is_empty(),
                    "{file_name}: item label should not be empty"
                );
            }
        }

        // Every visit is identified by its own code.
        for (index, visit) in project.visit.iter().enumerate() {
            assert!(
                !visit.code.is_empty(),
                "{file_name}: visit code should not be empty"
            );
            assert!(
                !project.visit[..index]
                    .iter()
                    .any(|other| other.code == visit.code),
                "{file_name}: visit codes should be unique"
            );
        }

        if let Some((forms, visits)) = expected_counts(file_name) {
            assert_eq!(
                project.forms.len(),
                forms,
                "{file_name}: unexpected number of forms"
            );
            assert_eq!(
                project.visit.len(),
                visits,
                "{file_name}: unexpected number of visits"
            );
        }
    }
}

#[test]
fn test_parse_rave_als_from_reader() {
    let Some(path) = mock_rave_files().into_iter().next() else {
        eprintln!("Skipping - .mock_data/als/rave not found");
        return;
    };

    let bytes = std::fs::read(&path).unwrap();
    let project = parse_rave_als(std::io::Cursor::new(bytes)).unwrap();

    assert!(!project.forms.is_empty(), "Project should have forms");
    assert!(!project.visit.is_empty(), "Project should have visits");
}

/// The workbooks use numeric character references (`&#13;`, `&#10;`) for the
/// line breaks of multi line field labels. They have to survive the parse.
#[test]
fn test_parse_rave_als_keeps_numeric_character_references() {
    let Some(path) = mock_rave_files()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|name| name == "rave01.xml"))
    else {
        eprintln!("Skipping - .mock_data/als/rave/rave01.xml not found");
        return;
    };

    let project = parse_rave_als(std::fs::File::open(&path).unwrap()).unwrap();

    let labels: Vec<&str> = project
        .forms
        .iter()
        .flat_map(|form| form.items.iter())
        .map(|item| item.label.as_str())
        .collect();

    assert!(
        labels.iter().any(|label| label.contains("\r\n")),
        "multi line labels should keep their line breaks"
    );
}
