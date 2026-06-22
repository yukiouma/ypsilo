# als-resolver

Parse ALS (Audit Landmark Study) files into a unified `Project` structure.

## Overview

`als-resolver` converts ALS exports from electronic data capture (EDC) systems into a normalized `Project` model suitable for further processing, analysis, or migration. It handles three distinct ALS formats:

| Format | System | File Type |
|--------|--------|-----------|
| Rave ALS | Medidata Rave | XML |
| ecollect v6 | Taimei eCollect V6 | XLSX |
| ecollect Legacy | Taimei eCollect | XLSX |

## Usage

```rust
use als_resolver::{parse_rave_als, parse_ecollect_v6_als, parse_ecollect_legacy_als};

let project = parse_rave_als("path/to/rave.xml")?;
let project = parse_ecollect_v6_als("path/to/ecollect_v6.xlsx")?;
let project = parse_ecollect_legacy_als("path/to/ecollect_legacy.xlsx")?;

println!("Forms: {}  Visits: {}", project.forms.len(), project.visit.len());
```

## Output Model

Parsing produces a `Project` containing:

```rust
pub struct Project {
    pub forms: Vec<CRFForm>,   // All CRF forms in the study
    pub visit: Vec<Visit>,     // All study visits
}

pub struct CRFForm {
    pub name: String,
    pub description: String,
    pub order: i32,
    pub items: Vec<CRFItem>,
    pub domains: Vec<Domain>,
    pub annotations: Vec<Annotation>,
}

pub struct CRFItem {
    pub name: String,
    pub label: String,
    pub item_option: Vec<ItemOption>,
    pub format: Option<String>,
    pub control_type: ControlType,
    pub item_unit: Option<ItemUnit>,
    pub not_variable: bool,
}

pub struct Visit {
    pub code: String,
    pub name: String,
    pub order: i32,
    pub forms: Vec<String>,  // OID references to forms at this visit
}
```

`ControlType` is an enum covering the common EDC control types: `TEXT`, `SELECTION`, `CHECKBOX`, `DATETIME`.

## Error Handling

All parse functions return `AlsParseError`:

```rust
pub enum AlsParseError {
    FileNotFound(String),
    IoError(String),
    XmlError(String),
    WorksheetNotFound(String),
    MissingRequiredField(String),
    InvalidFieldValue(String),
}
```

## Dependencies

The crate uses:
- `quick-xml` for Rave XML parsing
- `calamine` for ecollect XLSX parsing
- `entities` for the shared data model

Both parser implementations follow a multi-phase approach, reading reference data first (code lists, analytes, form definitions) before resolving form-item relationships and visit bindings.
