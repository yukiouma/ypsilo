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
}
