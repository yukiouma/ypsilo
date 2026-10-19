//! End-to-end scaffolding against the repository's own `config.json`.

use project_scaffold::{
    Context, ProjectScaffoldConfig, RawContext, ScaffoldError, project_scaffolding,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const CONFIG: &str = "/root/coding/project/ypsilo/.mock_data/project-scaffold/config.json";
const REFERENCES: &str = "/root/coding/project/ypsilo/.mock_data/project-scaffold/references";
const GENERAL_MARCOS: &str =
    "/root/coding/project/ypsilo/.mock_data/project-scaffold/references/general-marcos";

fn raw_context(root: &Path, project_root: Option<PathBuf>, config_path: PathBuf) -> RawContext {
    RawContext {
        product: "ak".to_string(),
        trial: "xx".to_string(),
        purpose: "final".to_string(),
        root_path: root.to_path_buf(),
        project_root,
        study: None,
        config_path,
    }
}

/// Scaffold the real config into `tmp`, rooted at `{tmp}/ak/xx`, study `AK-XX`.
fn scaffold(tmp: &TempDir) -> PathBuf {
    let context = Context::build(raw_context(tmp.path(), None, PathBuf::from(CONFIG))).unwrap();
    let root = context.project_root.clone();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();
    root
}

fn file_names(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn creates_the_expected_top_level_tree() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    assert!(root.join("documents").join("protocol").is_dir());
    assert!(root.join("documents").join("crf").is_dir());
    assert!(root.join("documents").join("sap").is_dir());
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
            .join("communication log")
            .is_dir()
    );
}

#[test]
fn interpolates_the_purpose_into_the_stats_tree() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    assert!(
        root.join("stats")
            .join("final")
            .join("product")
            .join("program")
            .join("adam")
            .is_dir()
    );
    assert!(root.join("stats").join("cutoff").join("rawdata").is_dir());
}

#[test]
fn copies_reference_files_under_their_interpolated_names() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    let sdtm = root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx");
    let template = Path::new(REFERENCES).join("AKxxx-xxx SDTM Specification_template v1.0.xlsx");
    assert_eq!(
        fs::metadata(&sdtm).unwrap().len(),
        fs::metadata(&template).unwrap().len()
    );

    let log = root
        .join("documents")
        .join("project management")
        .join("data issues tracking log")
        .join("AK-XX Data Issue Log.xlsx");
    assert!(log.is_file());
}

#[test]
fn migrates_only_the_sas_macros() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    let macros = root.join("stats").join("final").join("macros");
    let copied = file_names(&macros);

    let expected: BTreeSet<String> = fs::read_dir(GENERAL_MARCOS)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("sas")
        })
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    assert!(!expected.is_empty(), "fixture should contain .sas files");
    assert_eq!(copied, expected);
    assert!(!macros.join("Macro category information.xlsx").exists());
    assert!(!macros.join("update history.xlsx").exists());
}

#[test]
fn a_second_run_changes_nothing() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    let tracked = root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx");
    fs::write(&tracked, b"edited by hand").unwrap();
    let before = file_names(&root.join("stats").join("final").join("macros"));

    let context = Context::build(raw_context(
        tmp.path(),
        Some(root.clone()),
        PathBuf::from(CONFIG),
    ))
    .unwrap();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();

    assert_eq!(fs::read(&tracked).unwrap(), b"edited by hand");
    assert_eq!(
        before,
        file_names(&root.join("stats").join("final").join("macros"))
    );
}

#[test]
fn reports_a_missing_reference_file() {
    let tmp = TempDir::new().unwrap();
    let broken = tmp.path().join("broken.json");
    fs::write(
        &broken,
        r#"{"projectRoot": "{{ project_root }}", "directories": [
            {"name": "out.xlsx", "directory": false, "reference": "/nonexistent/template.xlsx"}
        ]}"#,
    )
    .unwrap();

    let context = Context::build(raw_context(tmp.path(), None, broken)).unwrap();
    let result = project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap());

    assert!(matches!(result, Err(ScaffoldError::ReferenceNotFound(_))));
}
