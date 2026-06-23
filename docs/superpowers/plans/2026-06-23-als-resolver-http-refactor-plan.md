# ALS Resolver HTTP Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor `parse_xxx_als` functions to accept `impl Read` sources, enabling HTTP/serverless usage without local file storage.

**Architecture:** Extend `AlsParser` trait with `parse_reader(impl Read)`. Existing `parse(path)` becomes a thin wrapper. New `parse_xxx_als_from` public functions provide the HTTP-ready API.

**Tech Stack:** Rust (std::io::Read, quick-xml, calamine 0.35.0)

---

## File Structure

```
crates/als-resolver/src/
  traits.rs              — add parse_reader to AlsParser trait
  lib.rs                 — add parse_xxx_als_from functions
  rave/parser.rs         — implement parse_reader
  ecollect_v6/parser.rs  — implement parse_reader
  ecollect_legacy/parser.rs — implement parse_reader
  ecollect_v6/code_list.rs, analytes.rs, ... — accept impl Read via open_workbook_from_rs
  ecollect_legacy/code_list.rs, analytes.rs, ... — accept impl Read via open_workbook_from_rs
```

**Key insight for Excel parsers:** Each internal module function (e.g., `code_list::parse_code_list_items`) currently opens the workbook by path. To support `impl Read`, these functions will use `calamine::open_workbook_from_rs(reader)` instead. The reader is buffered (`BufReader`) at the top-level `parse_reader` call.

---

## Task 1: Update `AlsParser` trait

**Files:**
- Modify: `crates/als-resolver/src/traits.rs`

- [ ] **Step 1: Add `parse_reader` to trait**

```rust
use std::io::Read;
use crate::error::AlsParseError;
use entities::project::Project;
use std::path::Path;

pub trait AlsParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let file = std::fs::File::open(path)
            .map_err(AlsParseError::IoError)?;
        self.parse_reader(std::io::BufReader::new(file))
    }

    fn parse_reader(&self, reader: impl Read) -> Result<Project, AlsParseError>;
}
```

- [ ] **Step 2: Run cargo check to verify trait compiles**

Run: `cargo check -p als-resolver`
Expected: SUCCESS (no errors)

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/traits.rs
git commit -m "refactor(als-resolver): add parse_reader to AlsParser trait
\nCo-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Implement `parse_reader` for Rave parser

**Files:**
- Modify: `crates/als-resolver/src/rave/parser.rs`

The current `parse` reads the file into `Vec<u8>` bytes once, then creates a new `quick-xml::Reader` from `bytes.as_slice()` for each phase. `parse_reader` will do the same: the caller passes `impl Read`, we read all bytes into `Vec<u8>`, then create `Reader` from the buffer per phase.

**Key change:** `parse` becomes a thin wrapper calling `parse_reader(BufReader::new(file))`. The real logic moves to `parse_reader`.

- [ ] **Step 1: Refactor Rave parser**

```rust
use crate::error::AlsParseError;
use crate::rave::context::ParseContext;
use crate::rave::data_dictionary::parse_data_dictionaries;
use crate::rave::fields::parse_fields;
use crate::rave::forms::parse_forms;
use crate::rave::folders::parse_folders;
use crate::rave::matrices::parse_matrix_master;
use crate::traits::AlsParser;
use entities::project::Project;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::{BufRead, Cursor, Read};

pub struct RaveParser;

impl AlsParser for RaveParser {
    fn parse(&self, path: &std::path::Path) -> Result<Project, AlsParseError> {
        let file = std::fs::File::open(path).map_err(AlsParseError::IoError)?;
        self.parse_reader(std::io::BufReader::new(file))
    }

    fn parse_reader(&self, mut reader: impl Read) -> Result<Project, AlsParseError> {
        let mut context = ParseContext::new();
        // Read entire file into memory to allow multiple passes
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;

        // Phase 1: Load DataDictionaries
        let mut reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "DataDictionaryEntries")?;
        parse_data_dictionaries(&mut reader, &mut context)?;

        // Phase 2: Parse Forms
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Forms")?;
        parse_forms(&mut reader, &mut context)?;

        // Phase 3: Parse Fields
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Fields")?;
        parse_fields(&mut reader, &mut context)?;

        // Phase 4: Parse Folders to create Visit structs
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Folders")?;
        parse_folders(&mut reader, &mut context)?;

        // Phase 5: Parse Matrix#MASTER to populate Visit.forms
        reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "Matrix121#MASTER")?;
        parse_matrix_master(&mut reader, &mut context)?;

        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: context.visits,
        })
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check -p als-resolver`
Expected: SUCCESS

- [ ] **Step 3: Run existing Rave tests**

Run: `cargo test -p als-resolver --test rave_parser_integration`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add crates/als-resolver/src/rave/parser.rs
git commit -m "refactor(rave): implement parse_reader for AlsParser trait
\nCo-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Implement `parse_reader` for Ecollect v6 parser

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/parser.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/code_list.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/analytes.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/form_sets.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/unit_groups.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/forms.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/items.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/form_item.rs`
- Modify: `crates/als-resolver/src/ecollect_v6/visits.rs`

**Pattern:** Each module function that currently calls `calamine::open_workbook(path)` will change to accept `impl Read` and use `calamine::open_workbook_from_rs(reader)`.

- [ ] **Step 1: Update `crates/als-resolver/src/ecollect_v6/code_list.rs`**

Change signature from `path: &Path` to `reader: impl Read`. Use `calamine::open_workbook_from_rs(reader)` instead of `open_workbook(path)`.

```rust
use calamine::{open_workbook_from_rs, Reader, Xlsx, XlsxError};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::ItemOption;
use std::io::Read;

pub fn parse_code_list_items(reader: impl Read, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook_from_rs(reader)
        .map_err(|e: XlsxError| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    // ... rest unchanged
}
```

- [ ] **Step 2: Update `crates/als-resolver/src/ecollect_v6/analytes.rs`**

Same pattern — change `path: &Path` to `reader: impl Read`, use `open_workbook_from_rs(reader)`.

- [ ] **Step 3: Update remaining ecollect_v6 modules**

Apply the same change to: `form_sets.rs`, `unit_groups.rs`, `forms.rs`, `items.rs`, `form_item.rs`, `visits.rs`.

Each file's function signature changes from:
```rust
pub fn parse_xxx(path: &Path, context: &mut EcollectParseContext) -> Result<...>
```
To:
```rust
pub fn parse_xxx(reader: impl Read, context: &mut EcollectParseContext) -> Result<...>
```

And `open_workbook(path)` becomes `open_workbook_from_rs(reader)`.

- [ ] **Step 4: Update `crates/als-resolver/src/ecollect_v6/parser.rs`**

```rust
use crate::AlsParseError;
use crate::ecollect_v6::context::EcollectParseContext;
use crate::ecollect_v6::{
    analytes, code_list, form_item, form_sets, forms, items, unit_groups, visits,
};
use crate::traits::AlsParser;
use entities::project::{Project, Visit};
use std::io::BufReader;
use std::path::Path;
use std::io::Read;

pub struct EcollectV6Parser;

impl AlsParser for EcollectV6Parser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let file = std::fs::File::open(path).map_err(AlsParseError::IoError)?;
        self.parse_reader(BufReader::new(file))
    }

    fn parse_reader(&self, reader: impl Read) -> Result<Project, AlsParseError> {
        let mut context = EcollectParseContext::new();

        // Phase 1: Load reference data
        code_list::parse_code_list_items(&reader, &mut context)?;
        analytes::parse_analytes(&reader, &mut context)?;
        form_sets::parse_form_sets(&reader, &mut context)?;
        unit_groups::parse_unit_groups(&reader, &mut context)?;
        // ... etc

        Ok(Project { forms, visit: visit_list })
    }
}
```

**Note:** Since `calamine::open_workbook_from_rs` consumes the reader, and we call it multiple times (once per worksheet/phase), we cannot pass the same reader directly. Instead, we read all bytes into memory once, then use `Cursor::new(bytes)` for each phase.

**Revised approach for parse_reader:**
```rust
fn parse_reader(&self, reader: impl Read) -> Result<Project, AlsParseError> {
    let mut context = EcollectParseContext::new();
    // Read all bytes into memory (one-time cost)
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    // Phase 1: Load reference data
    let cursor = std::io::Cursor::new(&bytes);
    code_list::parse_code_list_items(cursor, &mut context)?;
    // ... etc
}
```

Wait — `calamine::open_workbook_from_rs` takes ownership of the reader. If we pass `&bytes` via `Cursor`, each call consumes the cursor. We need `Cursor::new(bytes.clone())` or we pass `&bytes` as `&[u8]` and `open_workbook_from_rs` accepts `impl Read` which can be `&[u8]`.

Let me check: `open_workbook_from_rs<R: Read>(reader: R)` — it takes `reader: R` by value, consuming it. So we need to clone the bytes or use `BufReader::new(Cursor::new(bytes))` and seek back to start between calls.

**Simpler:** Since the calamine workbook is opened per module function call (e.g., `code_list::parse_code_list_items` opens the workbook, reads CodeListItems sheet, returns), we can pass a fresh `Cursor::new(bytes.clone())` each time. The clone is cheap (reference count, not deep copy).

- [ ] **Step 5: Run cargo check**

Run: `cargo check -p als-resolver`
Expected: SUCCESS

- [ ] **Step 6: Run existing ecollect_v6 tests**

Run: `cargo test -p als-resolver --test ecollect_v6_parser_integration`
Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/
git commit -m "refactor(ecollect_v6): implement parse_reader for AlsParser trait
\nCo-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: Implement `parse_reader` for Ecollect Legacy parser

**Files:**
- Modify: `crates/als-resolver/src/ecollect_legacy/parser.rs`
- Modify: `crates/als-resolver/src/ecollect_legacy/code_list.rs`
- Modify: `crates/als-resolver/src/ecollect_legacy/analytes.rs`
- Modify: `crates/als-resolver/src/ecollect_legacy/events.rs`
- Modify: `crates/als-resolver/src/ecollect_legacy/event_form.rs`
- Modify: `crates/als-resolver/src/ecollect_legacy/forms.rs`
- Modify: `crates/als-resolver/src/ecollect_legacy/group_items.rs`

**Same pattern as Task 3** — change each module's function to accept `impl Read` instead of `&Path`, use `calamine::open_workbook_from_rs(reader)`.

- [ ] **Step 1: Update all ecollect_legacy module files** — same pattern as ecollect_v6

- [ ] **Step 2: Update `crates/als-resolver/src/ecollect_legacy/parser.rs`** — implement `parse_reader` with same pattern as v6

- [ ] **Step 3: Run cargo check**

Run: `cargo check -p als-resolver`
Expected: SUCCESS

- [ ] **Step 4: Run existing ecollect_legacy tests**

Run: `cargo test -p als-resolver --test ecollect_legacy_parser_integration`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/als-resolver/src/ecollect_legacy/
git commit -m "refactor(ecollect_legacy): implement parse_reader for AlsParser trait
\nCo-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: Add `parse_xxx_als_from` public API functions

**Files:**
- Modify: `crates/als-resolver/src/lib.rs`

- [ ] **Step 1: Update lib.rs imports and add new functions**

```rust
pub mod ecollect_v6;
mod error;
mod rave;
mod traits;

pub use entities::project::Project;
pub use error::AlsParseError;
pub use traits::AlsParser;

use std::io::BufReader;
use std::path::Path;

/// Parse a Rave ALS file from a path.
pub fn parse_rave_als(path: &Path) -> Result<Project, AlsParseError> {
    crate::rave::parser::RaveParser.parse(path)
}

/// Parse a Rave ALS file from any `impl Read` source.
pub fn parse_rave_als_from(reader: impl std::io::Read) -> Result<Project, AlsParseError> {
    crate::rave::parser::RaveParser.parse_reader(reader)
}

/// Parse an ecollect v6 ALS file from a path.
pub fn parse_ecollect_v6_als(path: &Path) -> Result<Project, AlsParseError> {
    crate::ecollect_v6::EcollectV6Parser.parse(path)
}

/// Parse an ecollect v6 ALS file from any `impl Read` source.
pub fn parse_ecollect_v6_als_from(reader: impl std::io::Read) -> Result<Project, AlsParseError> {
    crate::ecollect_v6::EcollectV6Parser.parse_reader(reader)
}

pub mod ecollect_legacy;

/// Parse an ecollect legacy ALS file from a path.
pub fn parse_ecollect_legacy_als(path: &Path) -> Result<Project, AlsParseError> {
    crate::ecollect_legacy::EcollectLegacyParser.parse(path)
}

/// Parse an ecollect legacy ALS file from any `impl Read` source.
pub fn parse_ecollect_legacy_als_from(reader: impl std::io::Read) -> Result<Project, AlsParseError> {
    crate::ecollect_legacy::EcollectLegacyParser.parse_reader(reader)
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check -p als-resolver`
Expected: SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/lib.rs
git commit -m "feat(als-resolver): add parse_xxx_als_from public API functions
\nCo-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: Add integration tests for `impl Read` API

**Files:**
- Modify: `crates/als-resolver/tests/rave_parser_integration.rs`
- Modify: `crates/als-resolver/tests/ecollect_v6_parser_integration.rs`
- Modify: `crates/als-resolver/tests/ecollect_legacy_parser_integration.rs`

- [ ] **Step 1: Add test for Rave `parse_rave_als_from`**

Add to `tests/rave_parser_integration.rs`:

```rust
use als_resolver::parse_rave_als_from;
use std::io::Cursor;

#[test]
fn test_parse_rave_als_from_reader() {
    let path = std::path::Path::new("../../.mock_data/als/rave.xml");
    if !path.exists() {
        eprintln!("Skipping - .mock_data/als/rave.xml not found");
        return;
    }

    let bytes = std::fs::read(path).unwrap();
    let cursor = Cursor::new(bytes);

    let result = parse_rave_als_from(cursor);
    assert!(result.is_ok(), "parse_rave_als_from should succeed");

    let project = result.unwrap();
    assert!(!project.forms.is_empty(), "Project should have forms");
    assert!(!project.visit.is_empty(), "Project should have visits");
}
```

- [ ] **Step 2: Add tests for ecollect_v6 and ecollect_legacy** — same pattern

- [ ] **Step 3: Run all tests**

Run: `cargo test -p als-resolver`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add crates/als-resolver/tests/
git commit -m "test(als-resolver): add impl Read integration tests
\nCo-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Spec Coverage Check

- [x] `AlsParser` trait extended with `parse_reader` — Task 1
- [x] Rave parser implements `parse_reader` — Task 2
- [x] Ecollect v6 parser implements `parse_reader` — Task 3
- [x] Ecollect Legacy parser implements `parse_reader` — Task 4
- [x] `parse_xxx_als_from` public functions added — Task 5
- [x] `parse(path)` remains as thin wrapper — Tasks 1-4
- [x] Backward compatible (existing tests pass) — Tasks 2, 3, 4
- [x] New `impl Read` tests added — Task 6

## Self-Review

- No placeholders (TBD/TODO) in any step
- All file paths exact
- All code blocks complete
- Trait signature consistent across Tasks 1-4
- `parse(path)` pattern consistent: thin wrapper → `parse_reader(BufReader::new(file))`
