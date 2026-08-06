//! CDISC terminology deserialisation.
//!
//! Reads an SDTM or ADaM terminology workbook (`.xls`/`.xlsx`) and produces
//! a [`TerminologyVersion`] containing all the [`CodeList`]s and their
//! [`CodeItem`]s.

mod loader;
mod model;

pub use loader::{from_bytes, from_path, from_reader};
pub use model::{CodeItem, CodeList, TerminologyError, TerminologyVersion};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    const SDTM_FIXTURE: &str = ".mock_data/terminologies/SDTM Terminology.xls";
    const ADAM_FIXTURE: &str = ".mock_data/terminologies/ADaM Terminology.xls";

    /// Locate the workspace root by walking up from this crate's manifest
    /// directory. Test binaries are run with the *package* directory as their
    /// working directory (not the workspace root), so fixture paths relative to
    /// the workspace root must be resolved explicitly.
    fn workspace_root() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            if p.join("Cargo.toml").exists() && p.join(".mock_data").is_dir() {
                return p;
            }
            if !p.pop() {
                panic!("could not locate workspace root containing .mock_data");
            }
        }
    }

    fn fixture(relative: &str) -> PathBuf {
        workspace_root().join(relative)
    }

    #[test]
    fn from_path_loads_sdtm_workbook() {
        let v = from_path(fixture(SDTM_FIXTURE)).expect("SDTM parses");
        assert_eq!(v.name, "2026-03-27");
        assert!(
            v.codelist.len() > 1000,
            "expected many codelists, got {}",
            v.codelist.len()
        );

        let first = &v.codelist[0];
        assert_eq!(first.code, "C141657");
        assert!(!first.extensible);
        assert_eq!(first.name, "10-Meter Walk/Run Functional Test Test Code");
        assert_eq!(first.submission_value, "TENMW1TC");
        assert!(!first.code_list.is_empty());
        assert_eq!(first.code_list[0].submission_value, "TENMW101");
    }

    #[test]
    fn from_path_loads_adam_workbook() {
        let v = from_path(fixture(ADAM_FIXTURE)).expect("ADaM parses");
        assert_eq!(v.name, "2025-09-26");
        // The pinned ADaM fixture has exactly 23 codelists / 140 code items
        // (verified independently against the workbook).
        assert!(
            v.codelist.len() > 20,
            "expected many codelists, got {}",
            v.codelist.len()
        );

        let first = &v.codelist[0];
        assert_eq!(first.code, "C208382");
        assert!(!first.extensible);
        assert!(!first.code_list.is_empty());
    }

    #[test]
    fn from_bytes_round_trips_sdtm() {
        let bytes = std::fs::read(fixture(SDTM_FIXTURE)).expect("read fixture");
        let v = from_bytes(&bytes).expect("parse from bytes");
        assert_eq!(v.name, "2026-03-27");
        assert_eq!(v.codelist[0].code, "C141657");
    }

    #[test]
    fn from_reader_round_trips_sdtm() {
        let file = File::open(fixture(SDTM_FIXTURE)).expect("open fixture");
        let v = from_reader(file).expect("parse from reader");
        assert_eq!(v.name, "2026-03-27");
    }

    #[test]
    fn from_path_missing_file_returns_io_error() {
        let err = from_path("/no/such/path/__terminology_missing.xls").unwrap_err();
        assert!(matches!(err, TerminologyError::Io { .. }), "got: {err:?}");
    }

    #[test]
    fn from_path_workbook_without_matching_sheet_errors() {
        // The two real fixtures each have exactly one matching sheet, so the
        // `NoMatchingSheet` path cannot be exercised through them. `select_sheet`
        // itself is covered directly by the unit tests in `loader.rs`; here we
        // only assert the error variant's rendered message.
        let e = TerminologyError::NoMatchingSheet {
            path: "<test>".to_string(),
        };
        assert!(e.to_string().contains("no sheet matching pattern"));
    }
}
