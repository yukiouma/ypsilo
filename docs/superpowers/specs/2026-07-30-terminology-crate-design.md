# Terminology Crate — Design

- **Date:** 2026-07-30
- **Status:** Approved (brainstorming)
- **Owner:** ypsilo / terminology

## 1. Goal

Implement the `terminology` crate at `crates/terminology/` per the requirements in
[`docs/terminology/terminology.md`](../../terminology/terminology.md):

1. Define the SDTM/ADaM terminology data model (`TerminologyVersion`, `CodeList`, `CodeItem`).
2. Deserialise a CDISC terminology workbook (`.xls`) into that model using the `calamine` crate.
3. Use `thiserror` for the crate's error type, `serde` for derived traits, and stay on `edition = "2024"`.
4. Export both the model and the deserialise functions from the crate root.

## 2. Crate layout

```
crates/terminology/
├── Cargo.toml          # calamine, thiserror, serde (from workspace)
├── src/
│   ├── lib.rs          # re-exports + thin entry points (from_path, from_reader, from_bytes)
│   ├── model.rs        # TerminologyVersion, CodeList, CodeItem, TerminologyError
│   └── loader.rs       # pure parse logic over a calamine::Range
```

Splitting I/O (entry points) from parsing lets unit tests feed a hand-built `calamine::Range` into the
core logic without touching the filesystem.

## 3. Public API

```rust
// model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyVersion {
    pub name: String,            // the yyyy-mm-dd suffix from the matched sheet name
    pub codelist: Vec<CodeList>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeList {
    pub code: String,
    pub extensible: bool,
    pub name: String,
    pub submission_value: String,
    pub synonym: String,
    pub definition: String,
    pub nci_preferred_term: String,
    pub code_list: Vec<CodeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeItem {
    pub code: String,
    pub submission_value: String,
    pub synonym: String,
    pub definition: String,
    pub nci_preferred_term: String,
}

#[derive(Debug, Error)]
pub enum TerminologyError { /* see §4 */ }

// lib.rs re-exports the three structs, the error, and three entry points:

pub fn from_path<P: AsRef<Path>>(path: P) -> Result<TerminologyVersion, TerminologyError>;
pub fn from_reader<R: Read + Seek>(reader: R) -> Result<TerminologyVersion, TerminologyError>;
pub fn from_bytes(bytes: &[u8]) -> Result<TerminologyVersion, TerminologyError>;
```

`Serialize`/`Deserialize` are derived to mirror the `checklog` crate's `LogResult`, so callers (and
tests) can round-trip the structure via JSON.

## 4. Error model

All variants carry `sheet` (the offending sheet name) where applicable, plus a 1-indexed `row` so the
caller can pinpoint the problem. Strict mode is the agreed policy: the first anomaly is reported and
parsing stops.

```rust
#[derive(Debug, Error)]
pub enum TerminologyError {
    #[error("I/O error reading {path}: {source}")]
    Io { path: String, #[source] source: std::io::Error },

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
    InvalidExtensible { sheet: String, row: usize, value: String },

    #[error("sheet '{sheet}' row {row}: CodeItem references unknown codelist code '{codelist_code}'")]
    OrphanCodeItem { sheet: String, row: usize, codelist_code: String },

    #[error("sheet '{sheet}' row {row}: {message}")]
    BadRow { sheet: String, row: usize, message: String },
}
```

## 5. Loader algorithm

For every `from_*` entry point:

1. Open the workbook with `calamine::open_workbook_auto` (handles `.xls` and `.xlsx`).
2. Scan sheet names against the regex
   `^(?P<prefix>.+) Terminology (?P<date>\d{4}-\d{2}-\d{2})$`.
3. Zero matches → `NoMatchingSheet { path }`. Multiple matches → `AmbiguousSheet { path, names }`.
4. Set `name = <date>` (the captured `yyyy-mm-dd`).
5. Read the matched sheet as a `Range`. Iterate rows starting at index 1 (header is index 0).
6. Maintain a `HashMap<String, usize>` from codelist code → index into the output `Vec<CodeList>`.
7. For each data row, validate column 0 (Code) is non-empty → `EmptyCode` on miss. Then:
   - If column 1 (Codelist Code) is empty:
     - Treat the row as a new `CodeList`.
     - Parse column 2 (Extensible) case-insensitively against `"Yes"`/`"No"` → `InvalidExtensible` on miss.
     - Push the new codelist and record `code → index` in the map.
   - Else:
     - Look up the codelist code in the map. If absent → `OrphanCodeItem`.
     - Otherwise append a `CodeItem` populated from columns 0, 4, 5, 6, 7.

Cell → `String` conversion handles `Data::String` (passed through, trimmed), `Data::Int`/`Data::Float`
(rendered via `to_string`), and `Data::Empty` (→ `""`). Anything else (`DateTime`, `Bool`, `Error`,
…) → `BadRow`.

## 6. Dependencies

In `crates/terminology/Cargo.toml` add to `[dependencies]`:

```toml
calamine = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
```

No `dev-dependencies` are required (tests construct `calamine::Range` values directly).

## 7. Testing

Two layers, both inside the crate:

- **Fixture-driven unit tests** (no I/O): build a `calamine::Range` from a small `Vec<Vec<Data>>`
  fixture and assert:
  - header row is skipped;
  - a CodeList row followed by CodeItem rows attaches the items to the right parent;
  - the second CodeList row starts a new parent;
  - `Yes`/`No` parsing handles mixed case and surrounding whitespace;
  - each strict-error variant fires on the right input (orphan item, missing sheet, ambiguous
    sheets, invalid extensible, empty code).

- **Real-file integration tests**: open
  `.mock_data/terminologies/SDTM Terminology.xls` and `.mock_data/terminologies/ADaM Terminology.xls`
  and assert:
  - `name == "2026-03-27"` (SDTM) / `"2025-09-26"` (ADaM);
  - the first codelist matches the inspection: `code == "C141657"`,
    `name == "10-Meter Walk/Run Functional Test Test Code"`, `extensible == false`;
  - the first attached code item has `submission_value == "TENMW1TC"` (SDTM) /
    `"APCH1TPS"` (ADaM first item);
  - `codelist.len() > 1000` for SDTM, `> 30` for ADaM;
  - the first 5 columns of `codelist[0]` are well-formed (no empty `code`, valid `extensible`).

The exploratory `examples/inspect_xls.rs` created during brainstorming will be removed before commit;
it served only to confirm the sheet/row layout.

## 8. Out of scope

- JSON serialisation is supported only via the derived `Serialize`/`Deserialize` traits; there is no
  separate writer.
- The crate exposes deserialisation only; no in-memory mutation API is planned.
- The `ReadMe` sheet is ignored entirely.
- Cells with `DateTime`, `Bool`, or `Error` data are rejected (`BadRow`) — they should not occur in
  CDISC terminology workbooks.

## 9. Open questions

None. Design is approved.
