use std::collections::HashMap;

use crate::model::{CodeItem, CodeList, TerminologyError, TerminologyVersion};
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

/// Parse every data row in `range` into a [`TerminologyVersion`].
///
/// `source` is the path or other human-readable identifier used in error
/// messages; `sheet_name` is the matched sheet name and is included in error
/// variants that carry a sheet context. Row numbers reported in errors are
/// 1-indexed and count the header row (so the first data row is row 2).
pub(crate) fn parse_range(
    _source: &str,
    sheet_name: &str,
    range: &calamine::Range<calamine::Data>,
) -> Result<TerminologyVersion, TerminologyError> {
    let mut codelists: Vec<CodeList> = Vec::new();
    let mut codelist_index: HashMap<String, usize> = HashMap::new();

    for (idx, row) in range.rows().enumerate() {
        let row_number = idx + 1; // 1-indexed, header is row 1

        // Skip the header row.
        if idx == 0 {
            continue;
        }

        // Pad short rows so missing trailing cells are treated as empty.
        let padded: Vec<calamine::Data> = (0..8)
            .map(|i| row.get(i).cloned().unwrap_or(calamine::Data::Empty))
            .collect();

        let cells: Vec<String> = padded
            .iter()
            .map(cell_to_string)
            .collect::<Result<_, _>>()
            .map_err(|message| TerminologyError::BadRow {
                sheet: sheet_name.to_string(),
                row: row_number,
                message,
            })?;

        let code = cells[0].clone();
        if code.is_empty() {
            return Err(TerminologyError::EmptyCode {
                sheet: sheet_name.to_string(),
                row: row_number,
            });
        }
        let codelist_code_ref = &cells[1];
        let extensible = &cells[2];
        let name = cells[3].clone();
        let submission_value = cells[4].clone();
        let synonym = cells[5].clone();
        let definition = cells[6].clone();
        let nci_preferred_term = cells[7].clone();

        if codelist_code_ref.is_empty() {
            // CodeList row.
            let ext = match extensible.to_ascii_lowercase().as_str() {
                "yes" => true,
                "no" => false,
                _ => {
                    return Err(TerminologyError::InvalidExtensible {
                        sheet: sheet_name.to_string(),
                        row: row_number,
                        // Report the value as it appears in the workbook, not
                        // the lowercased form used for matching.
                        value: extensible.clone(),
                    });
                }
            };
            let new_idx = codelists.len();
            codelist_index.insert(code.clone(), new_idx);
            codelists.push(CodeList {
                code,
                extensible: ext,
                name,
                submission_value,
                synonym,
                definition,
                nci_preferred_term,
                code_list: Vec::new(),
            });
        } else {
            // CodeItem row.
            let parent_idx = *codelist_index.get(codelist_code_ref).ok_or_else(|| {
                TerminologyError::OrphanCodeItem {
                    sheet: sheet_name.to_string(),
                    row: row_number,
                    codelist_code: codelist_code_ref.clone(),
                }
            })?;
            codelists[parent_idx].code_list.push(CodeItem {
                code,
                submission_value,
                synonym,
                definition,
                nci_preferred_term,
            });
        }
    }

    Ok(TerminologyVersion {
        name: String::new(), // populated by `parse_range_with_date`
        codelist: codelists,
    })
}

/// Like [`parse_range`], but fills the resulting [`TerminologyVersion`]'s
/// `name` field with `date`.
pub(crate) fn parse_range_with_date(
    source: &str,
    sheet_name: &str,
    date: &str,
    range: &calamine::Range<calamine::Data>,
) -> Result<TerminologyVersion, TerminologyError> {
    let mut v = parse_range(source, sheet_name, range)?;
    v.name = date.to_string();
    Ok(v)
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

#[cfg(test)]
mod parse_range_tests {
    use super::*;
    use calamine::Data;

    /// Build a 2-D `Vec<Vec<Data>>` fixture and turn it into a `Range<Data>`.
    /// Header row is index 0; data rows follow.
    ///
    /// NOTE: calamine 0.35 does not expose `Range::from_iter`; the closest
    /// equivalent is `Range::from_sparse(Vec<Cell<T>>)` which infers the
    /// range bounds from the supplied cells' positions and fills the
    /// remainder with `T::default()` (i.e. `Data::Empty`).
    fn range_from_rows(rows: Vec<Vec<Data>>) -> calamine::Range<Data> {
        let mut cells = Vec::new();
        for (r, row) in rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                cells.push(calamine::Cell::new((r as u32, c as u32), cell.clone()));
            }
        }
        calamine::Range::from_sparse(cells)
    }

    fn s(v: &str) -> Data {
        Data::String(v.to_string())
    }

    fn empty() -> Data {
        Data::Empty
    }

    fn sdtm_fixture() -> Vec<Vec<Data>> {
        vec![
            // Header row — must be skipped.
            vec![s("Code"), s("Codelist Code"), empty(), empty(), empty(), empty(), empty(), empty()],
            // CodeList 1
            vec![
                s("C141657"),
                empty(),
                s("No"),
                s("Ten-Meter Walk/Run Test Code"),
                s("TENMW1TC"),
                s("synA"),
                s("defA"),
                s("nciA"),
            ],
            // CodeItem 1 under C141657
            vec![
                s("C174106"),
                s("C141657"),
                empty(),
                empty(),
                s("TENMW101"),
                s("synB"),
                s("defB"),
                s("nciB"),
            ],
            // CodeItem 2 under C141657
            vec![
                s("C141700"),
                s("C141657"),
                empty(),
                empty(),
                s("TENMW102"),
                empty(),
                empty(),
                empty(),
            ],
            // CodeList 2
            vec![
                s("C141656"),
                empty(),
                s("Yes"),
                s("Ten-Meter Walk/Run Test Name"),
                s("TENMW1TN"),
                s("synC"),
                s("defC"),
                s("nciC"),
            ],
            // CodeItem under CodeList 2
            vec![
                s("C141701"),
                s("C141656"),
                empty(),
                empty(),
                s("TENMW1-Test Grade"),
                empty(),
                empty(),
                empty(),
            ],
        ]
    }

    #[test]
    fn parse_range_skips_header_and_groups_items() {
        let range = range_from_rows(sdtm_fixture());
        let v = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).expect("parse");

        // `name` is populated by the Task 6 wrapper; in this task it is empty.
        assert_eq!(v.name, "");
        assert_eq!(v.codelist.len(), 2);

        let cl0 = &v.codelist[0];
        assert_eq!(cl0.code, "C141657");
        assert!(!cl0.extensible);
        assert_eq!(cl0.name, "Ten-Meter Walk/Run Test Code");
        assert_eq!(cl0.submission_value, "TENMW1TC");
        assert_eq!(cl0.code_list.len(), 2);

        let item0 = &cl0.code_list[0];
        assert_eq!(item0.code, "C174106");
        assert_eq!(item0.submission_value, "TENMW101");
        assert_eq!(item0.synonym, "synB");

        let cl1 = &v.codelist[1];
        assert_eq!(cl1.code, "C141656");
        assert!(cl1.extensible);
        assert_eq!(cl1.code_list.len(), 1);
        assert_eq!(cl1.code_list[0].code, "C141701");
    }

    #[test]
    fn parse_range_trims_string_cells() {
        let range = range_from_rows(vec![
            vec![s("Code"), empty(), empty(), empty(), empty(), empty(), empty(), empty()],
            vec![
                s(" C1 "),
                empty(),
                s(" No "),
                s(" Name "),
                s(" SV "),
                s(" Syn "),
                s(" Def "),
                s(" NCI "),
            ],
        ]);
        let v = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).expect("parse");
        let cl = &v.codelist[0];
        assert_eq!(cl.code, "C1");
        assert_eq!(cl.extensible, false);
        assert_eq!(cl.name, "Name");
        assert_eq!(cl.submission_value, "SV");
        assert_eq!(cl.synonym, "Syn");
    }

    #[test]
    fn parse_range_handles_numeric_cells_in_text_columns() {
        // Some CDISC workbooks render the Code column as a numeric cell rather
        // than a string. The helper must accept either form.
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![
                Data::Int(254_467),
                empty(),
                s("No"),
                s("Test"),
                s("TST"),
                empty(),
                empty(),
                empty(),
            ],
        ]);
        let v = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).expect("parse");
        assert_eq!(v.codelist[0].code, "254467");
    }

    #[test]
    fn parse_range_wraps_unsupported_cell_in_bad_row_error() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![
                Data::Bool(true), // not a valid cell type
                empty(),
                s("No"),
                s("Test"),
                s("TST"),
                empty(),
                empty(),
                empty(),
            ],
        ]);
        let err = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).unwrap_err();
        match err {
            TerminologyError::BadRow { sheet, row, message } => {
                assert_eq!(sheet, "SDTM Terminology 2026-03-27");
                assert_eq!(row, 2); // 1-indexed; header is row 1, this is row 2
                assert!(message.contains("unsupported cell kind"), "got: {message}");
            }
            other => panic!("expected BadRow, got {other:?}"),
        }
    }

    // --- strict validations ----------------------------------------------------

    #[test]
    fn parse_range_rejects_empty_code_in_codelist_row() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![empty(), empty(), s("No"), s("N"), s("SV"), empty(), empty(), empty()],
        ]);
        let err = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).unwrap_err();
        assert!(matches!(err, TerminologyError::EmptyCode { row: 2, .. }));
    }

    #[test]
    fn parse_range_rejects_empty_code_in_codeitem_row() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            // Valid CodeList.
            vec![s("C1"), empty(), s("No"), s("N"), s("SV"), empty(), empty(), empty()],
            // CodeItem with empty Code column.
            vec![empty(), s("C1"), empty(), empty(), s("SV"), empty(), empty(), empty()],
        ]);
        let err = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).unwrap_err();
        assert!(matches!(err, TerminologyError::EmptyCode { row: 3, .. }));
    }

    #[test]
    fn parse_range_rejects_unparseable_extensible() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![s("C1"), empty(), s("Maybe"), s("N"), s("SV"), empty(), empty(), empty()],
        ]);
        let err = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).unwrap_err();
        match err {
            TerminologyError::InvalidExtensible { sheet, row, value } => {
                assert_eq!(sheet, "SDTM Terminology 2026-03-27");
                assert_eq!(row, 2);
                assert_eq!(value, "Maybe");
            }
            other => panic!("expected InvalidExtensible, got {other:?}"),
        }
    }

    #[test]
    fn parse_range_accepts_mixed_case_extensible() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![s("C1"), empty(), s("YES"), s("N"), s("SV"), empty(), empty(), empty()],
            vec![s("C2"), empty(), s("no"), s("N"), s("SV"), empty(), empty(), empty()],
        ]);
        let v = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).unwrap();
        assert!(v.codelist[0].extensible);
        assert!(!v.codelist[1].extensible);
    }

    #[test]
    fn parse_range_rejects_orphan_codeitem() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![s("C1"), empty(), s("No"), s("N"), s("SV"), empty(), empty(), empty()],
            vec![s("CI"), s("C999"), empty(), empty(), s("SV"), empty(), empty(), empty()],
        ]);
        let err = parse_range("src.xls", "SDTM Terminology 2026-03-27", &range).unwrap_err();
        match err {
            TerminologyError::OrphanCodeItem { sheet, row, codelist_code } => {
                assert_eq!(sheet, "SDTM Terminology 2026-03-27");
                assert_eq!(row, 3);
                assert_eq!(codelist_code, "C999");
            }
            other => panic!("expected OrphanCodeItem, got {other:?}"),
        }
    }

    #[test]
    fn parse_range_with_date_sets_name_field() {
        let range = range_from_rows(vec![
            vec![empty(); 8],
            vec![s("C1"), empty(), s("No"), s("N"), s("SV"), empty(), empty(), empty()],
        ]);
        let v = parse_range_with_date(
            "src.xls",
            "SDTM Terminology 2026-03-27",
            "2026-03-27",
            &range,
        )
        .unwrap();
        assert_eq!(v.name, "2026-03-27");
    }
}
