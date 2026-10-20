//! End-to-end scaffolding over a representative configuration tree.
//!
//! The fixtures are built inside a `TempDir` rather than read from `.mock_data/`,
//! which is gitignored — so these tests run on any machine, not just this checkout.

use project_scaffold::{
    Context, ProjectScaffoldConfig, RawContext, ScaffoldError, project_scaffolding,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

/// The fixture tree: a `documents` branch, a `stats` branch holding a `{{ purpose }}`
/// directory, and a macro migration that must pick `.sas` files and skip `.xlsx`.
fn fixture_config(references: &Path) -> serde_json::Value {
    json!({
        "projectRoot": "{{ project_root }}",
        "directories": [
            {
                "name": "documents",
                "directory": true,
                "children": [
                    { "name": "protocol", "directory": true },
                    { "name": "crf", "directory": true },
                    { "name": "{{ study | lowercase }} notes", "directory": true },
                    { "name": "{{ date }} log", "directory": true },
                    {
                        "name": "specs",
                        "directory": true,
                        "children": [
                            {
                                "name": "{{ study }} SDTM Specification v1.0.xlsx",
                                "directory": false,
                                "reference": references
                                    .join("SDTM Specification_template v1.0.xlsx")
                                    .display()
                                    .to_string()
                            },
                            {
                                "name": "{{ study }} ADaM Specification v1.0.xlsx",
                                "directory": false,
                                "reference": references
                                    .join("ADaM Specification_template v1.0.xlsx")
                                    .display()
                                    .to_string()
                            }
                        ]
                    },
                    {
                        "name": "project management",
                        "directory": true,
                        "children": [
                            {
                                "name": "data issues tracking log",
                                "directory": true,
                                "children": [
                                    {
                                        "name": "{{ study }} Data Issue Log.xlsx",
                                        "directory": false,
                                        "reference": references
                                            .join("Data Issue Log-V1.0.xlsx")
                                            .display()
                                            .to_string()
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            {
                "name": "stats",
                "directory": true,
                "children": [
                    {
                        "name": "{{ purpose }}",
                        "directory": true,
                        "children": [
                            {
                                "name": "product",
                                "directory": true,
                                "children": [
                                    {
                                        "name": "program",
                                        "directory": true,
                                        "children": [
                                            { "name": "adam", "directory": true },
                                            { "name": "sdtm", "directory": true }
                                        ]
                                    }
                                ]
                            },
                            {
                                "name": "utility",
                                "directory": true,
                                "children": [
                                    {
                                        "name": "{{ study }} Validation Review Tracker.xlsx",
                                        "directory": false,
                                        "reference": references
                                            .join("Validation Review Tracker-V2.0.xlsx")
                                            .display()
                                            .to_string()
                                    }
                                ]
                            },
                            {
                                "name": "macros",
                                "directory": true,
                                "migration": {
                                    "directory": references.join("general-marcos").display().to_string(),
                                    "filter": [".sas"]
                                }
                            }
                        ]
                    },
                    {
                        "name": "cutoff",
                        "directory": true,
                        "children": [{ "name": "rawdata", "directory": true }]
                    }
                ]
            }
        ]
    })
}

/// A fixture workspace: reference workbooks, a `general-marcos` source directory
/// holding two `.sas` files and two `.xlsx` files, and the config that uses them.
struct Fixture {
    _dir: TempDir,
    root: TempDir,
    config_path: PathBuf,
    macros_source: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let root = TempDir::new().unwrap();

        let references = dir.path().join("references");
        fs::create_dir(&references).unwrap();
        for name in [
            "SDTM Specification_template v1.0.xlsx",
            "ADaM Specification_template v1.0.xlsx",
            "Data Issue Log-V1.0.xlsx",
            "Validation Review Tracker-V2.0.xlsx",
        ] {
            fs::write(references.join(name), format!("contents of {name}")).unwrap();
        }

        let macros_source = references.join("general-marcos");
        fs::create_dir(&macros_source).unwrap();
        fs::write(macros_source.join("m-forest-v1-0.sas"), b"%macro forest").unwrap();
        // A real fixture has a space before the extension; the filter must still match it.
        fs::write(macros_source.join("m-seriesplot-v1-0 .sas"), b"%macro plot").unwrap();
        fs::write(
            macros_source.join("Macro category information.xlsx"),
            b"book",
        )
        .unwrap();
        fs::write(macros_source.join("update history.xlsx"), b"book").unwrap();

        let config_path = dir.path().join("config.json");
        fs::write(
            &config_path,
            serde_json::to_string_pretty(&fixture_config(&references)).unwrap(),
        )
        .unwrap();

        Self {
            _dir: dir,
            root,
            config_path,
            macros_source,
        }
    }

    fn raw_context(&self, project_root: Option<PathBuf>) -> RawContext {
        RawContext {
            product: "ak".to_string(),
            trial: "xx".to_string(),
            purpose: Some("final".to_string()),
            root_path: self.root.path().to_path_buf(),
            project_root,
            study: None,
            config_path: self.config_path.clone(),
        }
    }

    /// Scaffold into `self.root`, returning the resolved project root.
    fn scaffold(&self) -> PathBuf {
        let context = Context::build(self.raw_context(None)).unwrap();
        let project_root = context.project_root.clone();
        project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();
        project_root
    }
}

fn file_names(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn derives_the_project_root_under_the_root_path() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    assert_eq!(root, fixture.root.path().join("ak").join("xx"));
    assert!(root.is_dir());
}

#[test]
fn creates_the_nested_directory_tree() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    assert!(root.join("documents").join("protocol").is_dir());
    assert!(root.join("documents").join("crf").is_dir());
    assert!(root.join("documents").join("specs").is_dir());
    assert!(
        root.join("documents")
            .join("project management")
            .join("data issues tracking log")
            .is_dir()
    );
}

#[test]
fn interpolates_purpose_and_study_into_directory_and_file_names() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    // `{{ purpose }}` names a whole branch of the tree.
    assert!(
        root.join("stats")
            .join("final")
            .join("product")
            .join("program")
            .join("adam")
            .is_dir()
    );
    assert!(root.join("stats").join("cutoff").join("rawdata").is_dir());

    // `{{ study }}` names files, interpolated to the derived `AK-XX`.
    assert!(
        root.join("documents")
            .join("specs")
            .join("AK-XX SDTM Specification v1.0.xlsx")
            .is_file()
    );
    assert!(
        root.join("documents")
            .join("specs")
            .join("AK-XX ADaM Specification v1.0.xlsx")
            .is_file()
    );
    assert!(
        root.join("documents")
            .join("project management")
            .join("data issues tracking log")
            .join("AK-XX Data Issue Log.xlsx")
            .is_file()
    );
    assert!(
        root.join("stats")
            .join("final")
            .join("utility")
            .join("AK-XX Validation Review Tracker.xlsx")
            .is_file()
    );
}

#[test]
fn applies_a_template_method_to_a_directory_name() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    // `{{ study | lowercase }}` is `AK-XX` in lower case.
    assert!(root.join("documents").join("ak-xx notes").is_dir());
}

#[test]
fn stamps_the_current_date_into_a_directory_name() {
    let fixture = Fixture::new();
    let context = Context::build(fixture.raw_context(None)).unwrap();
    let date = context.date.clone();
    let project_root = context.project_root.clone();

    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();

    assert!(
        project_root
            .join("documents")
            .join(format!("{date} log"))
            .is_dir()
    );
}

#[test]
fn skips_the_purpose_branch_when_the_context_carries_no_purpose() {
    let fixture = Fixture::new();
    let mut raw = fixture.raw_context(None);
    raw.purpose = None;

    let context = Context::build(raw).unwrap();
    let project_root = context.project_root.clone();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();

    // The `{{ purpose }}` branch has no value to resolve to, so it is left out —
    // the rest of the tree is unaffected.
    assert!(project_root.join("stats").join("cutoff").is_dir());
    assert!(!project_root.join("stats").join("final").exists());
}

#[test]
fn copies_reference_files_under_their_configured_names() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    let copied = root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx");
    let source = fixture
        .macros_source
        .parent()
        .unwrap()
        .join("SDTM Specification_template v1.0.xlsx");

    assert_eq!(fs::read(&copied).unwrap(), fs::read(&source).unwrap());
}

#[test]
fn migrates_only_the_sas_macros() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    let macros = root.join("stats").join("final").join("macros");
    let copied = file_names(&macros);

    // Both `.sas` files are copied, including the one with a space before the extension.
    assert_eq!(
        copied,
        BTreeSet::from([
            "m-forest-v1-0.sas".to_string(),
            "m-seriesplot-v1-0 .sas".to_string(),
        ])
    );
    // The `.xlsx` files in the source directory are filtered out.
    assert!(!macros.join("Macro category information.xlsx").exists());
    assert!(!macros.join("update history.xlsx").exists());
}

#[test]
fn a_second_run_changes_nothing() {
    let fixture = Fixture::new();
    let root = fixture.scaffold();

    let tracked = root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx");
    fs::write(&tracked, b"edited by hand").unwrap();
    let macros_before = file_names(&root.join("stats").join("final").join("macros"));

    let context = Context::build(fixture.raw_context(Some(root.clone()))).unwrap();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();

    // Neither a copied reference nor a migrated macro is overwritten.
    assert_eq!(fs::read(&tracked).unwrap(), b"edited by hand");
    assert_eq!(
        macros_before,
        file_names(&root.join("stats").join("final").join("macros"))
    );
}

#[test]
fn reuses_an_explicit_existing_project_root() {
    let fixture = Fixture::new();
    let explicit = fixture.root.path().join("somewhere-else");
    fs::create_dir(&explicit).unwrap();

    let context = Context::build(fixture.raw_context(Some(explicit.clone()))).unwrap();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();

    assert!(explicit.join("documents").is_dir());
    assert!(!fixture.root.path().join("ak").exists());
}

#[test]
fn reports_a_missing_reference_file() {
    let dir = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    let config_path = dir.path().join("broken.json");
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&json!({
            "projectRoot": "{{ project_root }}",
            "directories": [
                {
                    "name": "out.xlsx",
                    "directory": false,
                    "reference": "/nonexistent/template.xlsx"
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let context = Context::build(RawContext {
        product: "ak".to_string(),
        trial: "xx".to_string(),
        purpose: Some("final".to_string()),
        root_path: root.path().to_path_buf(),
        project_root: None,
        study: None,
        config_path,
    })
    .unwrap();

    let result = project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap());
    assert!(matches!(result, Err(ScaffoldError::ReferenceNotFound(_))));
}

#[test]
fn reports_a_missing_migration_directory() {
    let dir = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    let config_path = dir.path().join("broken.json");
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&json!({
            "projectRoot": "{{ project_root }}",
            "directories": [
                {
                    "name": "macros",
                    "directory": true,
                    "migration": {
                        "directory": "/nonexistent/general-marcos",
                        "filter": [".sas"]
                    }
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let context = Context::build(RawContext {
        product: "ak".to_string(),
        trial: "xx".to_string(),
        purpose: Some("final".to_string()),
        root_path: root.path().to_path_buf(),
        project_root: None,
        study: None,
        config_path,
    })
    .unwrap();

    let result = project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap());
    assert!(matches!(
        result,
        Err(ScaffoldError::MigrationDirNotFound(_))
    ));
}
