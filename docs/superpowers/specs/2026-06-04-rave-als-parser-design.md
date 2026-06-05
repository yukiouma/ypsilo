# Rave ALS to CRFForm Parser — Design Specification

**Date:** 2026/06/04
**Author:** Claude
**Status:** Draft

---

## Overview

Implement a feature in `crates/als-resolver` that parses a Medidata Rave EDC ALS (Excel XML) file into `Project` from `crates/entities`.

---

## Architecture

### Module Structure

```
als-resolver/src/
├── lib.rs              # Public API: parse_rave_als(path) -> Project
├── error.rs            # AlsParseError
├── traits.rs           # AlsParser trait
├── rave.rs             # pub mod parser, context, worksheet, crf_draft, forms, fields, folders, data_dictionary, matrices
└── rave/
    ├── parser.rs       # RaveParser struct + impl AlsParser
    ├── context.rs      # Shared parsing context (lookups)
    ├── worksheet.rs    # Worksheet navigation utilities
    ├── crf_draft.rs    # CRFDraft parsing
    ├── forms.rs        # Forms parsing
    ├── fields.rs       # Fields parsing + DataDictionary resolution
    ├── folders.rs      # Folders parsing
    ├── data_dictionary.rs  # DataDictionaries + DataDictionaryEntries
    └── matrices.rs     # Matrices + Matrix sheets parsing (Visit extraction)
```

### Key Design Decisions

1. **AlsParser trait** — Extensibility for future ALS formats (ecollect, etc.). Rave is one implementation.
2. **No mod.rs** — 2024 edition file-hierarchy style (module declares at parent, content in sibling file)
3. **Fail fast** — Return error on first parsing problem, no partial results
4. **Empty domains/annotations** — Per spec, `domains` and `annotations` remain empty `Vec`
5. **quick-xml streaming** — Memory efficient for 178MB+ files

---

## API

```rust
// traits.rs
pub trait AlsParser {
    fn parse(source: impl Read + 'static) -> Result<Project, AlsParseError>;
}

// lib.rs
pub fn parse_rave_als(path: &Path) -> Result<Project, AlsParseError> {
    let file = File::open(path).map_err(|e| AlsParseError::IoError(e.to_string()))?;
    RaveParser.parse(file)
}

pub fn parse_rave_als_stream(input: impl Read + 'static) -> Result<Project, AlsParseError> {
    RaveParser.parse(input)
}
```

**Returns:** `Project` containing:
- `forms: Vec<CRFForm>` — parsed forms with items
- `visit: Vec<Visit>` — parsed visits with form bindings from Matrix sheets

---

## Parsing Flow

### Phase 1: Load DataDictionaries (prerequisite for Fields)

1. Navigate to `DataDictionaries` worksheet
2. Parse all rows into `DataDictionary { name, oid }`
3. Navigate to `DataDictionaryEntries` worksheet
4. Parse all rows into `DataDictionaryEntry { dictionary_name, coded_data, ordinal, user_data_string, specify }`
5. Build lookup: `dictionary_name -> Vec<DataDictionaryEntry>`

### Phase 2: Parse Forms

1. Navigate to `Forms` worksheet
2. For each row, parse `CRFForm { name: DraftFormName, description: "", order: Ordinal, items: Vec::new(), domains: Vec::new(), annotations: Vec::new() }`
3. Store in context by OID

### Phase 3: Parse Fields (with DataDictionary resolution)

1. Navigate to `Fields` worksheet
2. For each field row:
   - Look up `DataDictionaryName` in context
   - If found, transform `DataDictionaryEntry` → `ItemOption`
   - Build `CRFItem` with `item_option: Some(options)` or `None`
3. Add items to corresponding form (by `FormOID`)

### Phase 4: Parse Folders

1. Navigate to `Folders` worksheet
2. Parse folder structure (for potential future domain derivation)
3. Store for reference

### Phase 5: Parse Visits (Matrices + Matrix sheets)

1. Navigate to `Matrices` worksheet
2. For each Matrix row, create `Visit { code: OID, name: MatrixName, order: Ordinal (or index), forms: Vec::new() }`
3. For each Matrix sheet (e.g., `Matrix1#C1`, `Matrix2#C11`):
   - Parse the sheet to extract form OIDs bound to this visit
   - Each row in a Matrix sheet represents a form-field mapping
   - Extract form OID from the row data (the "Matrix: {OID}" column or similar)
   - Add form OIDs to the corresponding Visit's `forms` field
4. Return `Vec<Visit>` alongside `Vec<CRFForm>`

**Note:** Matrix sheets encode repeating form structure. Parsing extracts which forms are bound to which visit windows (C1, C2, ... C41 columns per cycle).

---

## XML Structure (Excel SSXML)

```xml
<Worksheet ss:Name="Forms">
  <Table>
    <Column ss:Width="150"/>
    <Row>
      <Cell ss:StyleID="ColumnCaption"><Data ss:Type="String">OID</Data></Cell>
      <Cell ss:StyleID="ColumnCaption"><Data ss:Type="String">Ordinal</Data></Cell>
      ...
    </Row>
    <Row>
      <Cell ss:StyleID="Protected"><Data ss:Type="String">SC</Data></Cell>
      <Cell ss:StyleID="Default"><Data ss:Type="String">1</Data></Cell>
      ...
    </Row>
  </Table>
</Worksheet>
```

**Key patterns:**
- Header row with `ColumnCaption` style
- Data rows with `Protected` (OID) or `Default` style
- Cell index can skip columns via `ss:Index="N"`
- Data values in `<Data ss:Type="String">value</Data>`

---

## CRFForm Mapping

| CRFForm field | Source |
|---------------|--------|
| `name` | Forms.OID |
| `description` | Forms.DraftFormName |
| `order` | Forms.Ordinal → i32 |
| `items` | Fields rows (grouped by FormOID) |
| `domains` | Vec::new() |
| `annotations` | Vec::new() |

---

## CRFItem Mapping

| CRFItem field | Source |
|---------------|--------|
| `name` | Fields.FieldOID |
| `label` | Fields.DraftFieldName or Fields.PreText |
| `format` | Fields.DataFormat |
| `control_type` | Fields.ControlType → ControlType enum |
| `item_option` | DataDictionaryEntries (if DataDictionaryName set) |
| `item_unit` | Fields.FixedUnit → ItemUnit (if set) |
| `annotations` | Vec::new() |
| `not_variable` | None (not in source) |

### ControlType mapping

| XML Value | ControlType |
|-----------|-------------|
| "Text" | TEXT |
| "Select" | SELECTION |
| "Check" | CHECKBOX |
| "Radio" | SELECTION (mapped) |
| "File" | TEXT (fallback) |
| other | TEXT (fallback) |

---

## Error Handling

- **Fail fast** — first error stops parsing, returns `AlsParseError`
- `AlsParseError` variants:
  - `FileNotFound(String)` — path doesn't exist
  - `IoError(String)` — file read or stream error
  - `XmlError(String)` — quick-xml parsing error
  - `WorksheetNotFound(String)` — required sheet missing
  - `MissingRequiredField(String)` — OID, Ordinal, etc. missing
  - `InvalidFieldValue(String)` — malformed data

---

## Dependencies

Add to `crates/als-resolver/Cargo.toml`:
- `quick-xml = "..."` — streaming XML parser
- `thiserror = "..."` — error enum derive
- `entities` — internal crate dependency

Add to workspace `Cargo.toml` under `[workspace.dependencies]`:
- `quick-xml`
- `thiserror`

---

## Testing Strategy

1. **Unit tests** per module — parse small XML snippets
2. **Integration test** — parse `.mock_data/als/rave.xml` and verify output structure
3. **Edge cases** — empty fields, missing dictionaries, special characters

---

## Files to Create/Modify

### New files
- `crates/als-resolver/src/error.rs`
- `crates/als-resolver/src/traits.rs`
- `crates/als-resolver/src/rave.rs`
- `crates/als-resolver/src/rave/parser.rs`
- `crates/als-resolver/src/rave/context.rs`
- `crates/als-resolver/src/rave/worksheet.rs`
- `crates/als-resolver/src/rave/crf_draft.rs`
- `crates/als-resolver/src/rave/forms.rs`
- `crates/als-resolver/src/rave/fields.rs`
- `crates/als-resolver/src/rave/folders.rs`
- `crates/als-resolver/src/rave/data_dictionary.rs`
- `crates/als-resolver/src/rave/matrices.rs`

### Modified files
- `crates/als-resolver/src/lib.rs` — add public API
- `crates/als-resolver/Cargo.toml` — add dependencies
- `crates/entities/src/lib.rs` — update to `pub mod project;`
- `Cargo.toml` — add workspace dependencies

### Dependencies on entities
- `crates/entities` — als-resolver depends on entities for `Project`, `CRFForm`, `Visit` types (all in project.rs)