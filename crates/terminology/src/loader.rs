use crate::model::TerminologyError;
use calamine::Data;

/// Convert a single [`calamine::Data`] cell into a [`String`].
///
/// Strings are trimmed. Numeric cells are rendered via `Display`. Empty cells
/// become `""`. All other cell kinds (`Bool`, `DateTime`, `Error`) are rejected
/// — terminology workbooks should never contain them.
fn cell_to_string(cell: &Data) -> Result<String, String> {
    match cell {
        Data::String(s) => Ok(s.trim().to_string()),
        Data::Float(f) => Ok(f.to_string()),
        Data::Int(i) => Ok(i.to_string()),
        Data::Empty => Ok(String::new()),
        other => Err(format!("unsupported cell kind: {other:?}")),
    }
}

const SHEET_KEYWORD: &str = " Terminology ";

/// If `sheet_name` ends with `" Terminology yyyy-mm-dd"`, return the date.
/// Otherwise return `None`.
fn extract_date_suffix(sheet_name: &str) -> Option<String> {
    let (_, tail) = sheet_name.split_once(SHEET_KEYWORD)?;
    if tail.len() != 10 {
        return None;
    }
    let bytes = tail.as_bytes();
    let is_digit = |i: usize| bytes[i].is_ascii_digit();
    let is_hyphen = |i: usize| bytes[i] == b'-';
    if !(is_digit(0) && is_digit(1) && is_digit(2) && is_digit(3)
        && is_hyphen(4)
        && is_digit(5) && is_digit(6)
        && is_hyphen(7)
        && is_digit(8) && is_digit(9))
    {
        return None;
    }
    Some(tail.to_string())
}

/// Pick the single sheet whose name matches the pattern, returning its name
/// and the extracted date.
fn select_sheet<'a>(
    sheet_names: &'a [String],
    source: &str,
) -> Result<(&'a str, String), TerminologyError> {
    let matches: Vec<&str> = sheet_names
        .iter()
        .filter_map(|name| extract_date_suffix(name).map(|_| name.as_str()))
        .collect();

    match matches.len() {
        0 => Err(TerminologyError::NoMatchingSheet {
            path: source.to_string(),
        }),
        1 => {
            let name = matches[0];
            let date = extract_date_suffix(name)
                .expect("matched name must have a valid date suffix");
            Ok((name, date))
        }
        _ => Err(TerminologyError::AmbiguousSheet {
            path: source.to_string(),
            names: matches.into_iter().map(String::from).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_is_trimmed() {
        assert_eq!(cell_to_string(&Data::String("  hi  ".into())).unwrap(), "hi");
    }

    #[test]
    fn empty_string_stays_empty() {
        assert_eq!(cell_to_string(&Data::String(String::new())).unwrap(), "");
    }

    #[test]
    fn int_is_rendered() {
        assert_eq!(cell_to_string(&Data::Int(42)).unwrap(), "42");
    }

    #[test]
    fn float_is_rendered() {
        assert_eq!(cell_to_string(&Data::Float(1.5)).unwrap(), "1.5");
    }

    #[test]
    fn empty_cell_becomes_empty_string() {
        assert_eq!(cell_to_string(&Data::Empty).unwrap(), "");
    }

    #[test]
    fn bool_is_rejected() {
        let err = cell_to_string(&Data::Bool(true)).unwrap_err();
        assert!(err.contains("unsupported cell kind"), "got: {err}");
    }

    #[test]
    fn error_cell_is_rejected() {
        let err = cell_to_string(&Data::Error(calamine::CellErrorType::Div0)).unwrap_err();
        assert!(err.contains("unsupported cell kind"), "got: {err}");
    }

    // --- extract_date_suffix ---------------------------------------------------

    #[test]
    fn extract_date_suffix_matches_well_formed_name() {
        assert_eq!(
            extract_date_suffix("SDTM Terminology 2026-03-27"),
            Some("2026-03-27".to_string())
        );
        assert_eq!(
            extract_date_suffix("ADaM Terminology 2025-09-26"),
            Some("2025-09-26".to_string())
        );
    }

    #[test]
    fn extract_date_suffix_handles_arbitrary_prefix() {
        assert_eq!(
            extract_date_suffix("Anything Goes Terminology 1999-12-31"),
            Some("1999-12-31".to_string())
        );
    }

    #[test]
    fn extract_date_suffix_rejects_missing_keyword() {
        assert_eq!(extract_date_suffix("SDTM 2026-03-27"), None);
        assert_eq!(extract_date_suffix("SDTM Glossary 2026-03-27"), None);
    }

    #[test]
    fn extract_date_suffix_rejects_malformed_date() {
        assert_eq!(extract_date_suffix("SDTM Terminology 2026-3-27"), None);
        assert_eq!(extract_date_suffix("SDTM Terminology 26-03-27"), None);
        assert_eq!(extract_date_suffix("SDTM Terminology 2026-03-27 "), None);
        assert_eq!(extract_date_suffix("SDTM Terminology 2026/03/27"), None);
    }

    #[test]
    fn extract_date_suffix_rejects_missing_date() {
        assert_eq!(extract_date_suffix("SDTM Terminology"), None);
    }

    // --- select_sheet ----------------------------------------------------------

    #[test]
    fn select_sheet_picks_single_match() {
        let names = vec![
            "ReadMe".to_string(),
            "SDTM Terminology 2026-03-27".to_string(),
        ];
        let (sheet, date) = select_sheet(&names, "/tmp/foo.xls").expect("one match");
        assert_eq!(sheet, "SDTM Terminology 2026-03-27");
        assert_eq!(date, "2026-03-27");
    }

    #[test]
    fn select_sheet_errors_when_none_match() {
        let names = vec!["ReadMe".to_string(), "Glossary".to_string()];
        let err = select_sheet(&names, "/tmp/foo.xls").unwrap_err();
        assert!(matches!(err, TerminologyError::NoMatchingSheet { .. }));
        if let TerminologyError::NoMatchingSheet { path } = err {
            assert_eq!(path, "/tmp/foo.xls");
        }
    }

    #[test]
    fn select_sheet_errors_when_multiple_match() {
        let names = vec![
            "SDTM Terminology 2026-03-27".to_string(),
            "SDTM Terminology 2025-01-01".to_string(),
        ];
        let err = select_sheet(&names, "/tmp/foo.xls").unwrap_err();
        match err {
            TerminologyError::AmbiguousSheet { path, names: matched } => {
                assert_eq!(path, "/tmp/foo.xls");
                assert_eq!(matched.len(), 2);
            }
            other => panic!("expected AmbiguousSheet, got {other:?}"),
        }
    }

    #[test]
    fn select_sheet_skips_invalid_date_suffix() {
        // Sheet name has the keyword but the date is malformed; it must be
        // skipped, not counted as a match (and not trigger InvalidDateSuffix,
        // which only fires if a date WAS claimed to match the pattern).
        let names = vec!["SDTM Terminology not-a-date".to_string()];
        let err = select_sheet(&names, "/tmp/foo.xls").unwrap_err();
        assert!(matches!(err, TerminologyError::NoMatchingSheet { .. }));
    }
}
