//! Terminology data model and the crate-wide [`TerminologyError`].

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A single CDISC terminology workbook (SDTM or ADaM) for one publication date.
///
/// `name` carries the `yyyy-mm-dd` date extracted from the matched sheet name;
/// `codelist` is the ordered list of [`CodeList`]s in workbook order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyVersion {
    /// `yyyy-mm-dd` suffix of the matched sheet name (e.g. `"2026-03-27"`).
    pub name: String,
    /// All codelists, in workbook order.
    pub codelist: Vec<CodeList>,
}

/// A CDISC codelist and the [`CodeItem`]s that belong to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeList {
    /// NCI C-code of the codelist itself (column 0 of its defining row).
    pub code: String,
    /// Whether sponsors may add new permissible values.
    pub extensible: bool,
    /// Human-readable codelist name (column 3 of the defining row).
    pub name: String,
    /// CDISC submission value (column 4 of the defining row).
    pub submission_value: String,
    /// Comma-separated synonyms (column 5 of the defining row).
    pub synonym: String,
    /// CDISC definition (column 6 of the defining row).
    pub definition: String,
    /// NCI preferred term (column 7 of the defining row).
    pub nci_preferred_term: String,
    /// Permissible values belonging to this codelist, in workbook order.
    pub code_list: Vec<CodeItem>,
}

/// A single permissible value inside a [`CodeList`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeItem {
    /// NCI C-code of this item (column 0 of the item's row).
    pub code: String,
    /// CDISC submission value (column 4 of the item's row).
    pub submission_value: String,
    /// Comma-separated synonyms (column 5 of the item's row).
    pub synonym: String,
    /// CDISC definition (column 6 of the item's row).
    pub definition: String,
    /// NCI preferred term (column 7 of the item's row).
    pub nci_preferred_term: String,
}

/// Errors returned by every [`crate::from_*`] entry point and by [`crate::loader`].
#[derive(Debug, Error)]
pub enum TerminologyError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("workbook error: {0}")]
    Workbook(#[from] calamine::Error),

    #[error("no sheet matching pattern '<prefix> Terminology <yyyy-mm-dd>' in {path}")]
    NoMatchingSheet { path: String },

    #[error("multiple sheets match the pattern in {path}: {names:?}")]
    AmbiguousSheet { path: String, names: Vec<String> },

    #[error("invalid date suffix in sheet name '{name}'")]
    InvalidDateSuffix { name: String },

    #[error("sheet '{sheet}' row {row}: empty Code column")]
    EmptyCode { sheet: String, row: usize },

    #[error("sheet '{sheet}' row {row}: unparseable Extensible value '{value}'")]
    InvalidExtensible {
        sheet: String,
        row: usize,
        value: String,
    },

    #[error("sheet '{sheet}' row {row}: CodeItem references unknown codelist code '{codelist_code}'")]
    OrphanCodeItem {
        sheet: String,
        row: usize,
        codelist_code: String,
    },

    #[error("sheet '{sheet}' row {row}: {message}")]
    BadRow {
        sheet: String,
        row: usize,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_version() -> TerminologyVersion {
        TerminologyVersion {
            name: "2026-03-27".to_string(),
            codelist: vec![CodeList {
                code: "C141657".to_string(),
                extensible: false,
                name: "10-Meter Walk/Run Functional Test Test Code".to_string(),
                submission_value: "TENMW1TC".to_string(),
                synonym: "10-Meter Walk/Run Functional Test Test Code".to_string(),
                definition: "10-Meter Walk/Run test code.".to_string(),
                nci_preferred_term: "CDISC Functional Test 10-Meter Walk/Run Test Code Terminology"
                    .to_string(),
                code_list: vec![CodeItem {
                    code: "C174106".to_string(),
                    submission_value: "TENMW101".to_string(),
                    synonym: "TENMW1-Was Walk/Run Performed".to_string(),
                    definition: "10-Meter Walk/Run - Was the 10-meter walk/run performed?".to_string(),
                    nci_preferred_term: "10-Meter Walk/Run - Was Walk/Run Performed".to_string(),
                }],
            }],
        }
    }

    #[test]
    fn structs_construct_and_compare_equal() {
        let v = sample_version();
        let same = sample_version();
        assert_eq!(v, same);
        assert_eq!(v.codelist.len(), 1);
        assert_eq!(v.codelist[0].code_list.len(), 1);
        assert!(!v.codelist[0].extensible);
    }

    #[test]
    fn error_display_contains_context() {
        let e = TerminologyError::EmptyCode {
            sheet: "SDTM Terminology 2026-03-27".to_string(),
            row: 7,
        };
        let msg = e.to_string();
        assert!(msg.contains("SDTM Terminology 2026-03-27"));
        assert!(msg.contains("row 7"));
        assert!(msg.contains("empty Code"));
    }
}