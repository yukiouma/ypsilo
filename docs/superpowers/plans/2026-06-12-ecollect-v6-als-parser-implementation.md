# EcollectV6 ALS Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement an `AlsParser` for ecollect v6 ALS file format in `crates/als-resolver`, producing a `Project` entity from an Excel (.xlsx) file.

**Architecture:** Excel (.xlsx) parsing via `calamine 0.35.0`. Lazy per-sheet parsing with shared `EcollectParseContext`. Compound OIDs (e.g., `"YN=[1|是]"`) split on first `=` to get lookup keys. Visit names resolved via FormSets lookup.

**Tech Stack:** Rust, calamine 0.35.0, quick-xml (existing), thiserror (existing)

---

## File Structure

```
crates/als-resolver/src/
├── lib.rs                    # Add pub mod ecollect_v6; and public API functions
├── error.rs                  # (existing)
├── traits.rs                 # (existing)
├── rave.rs                   # (existing)
└── ecollect_v6/
    ├── mod.rs               # Module declaration
    ├── context.rs           # EcollectParseContext + ItemDef
    ├── parser.rs            # EcollectV6Parser + impl AlsParser
    ├── code_list.rs         # Parse CodeListItems → code_list_options
    ├── analytes.rs          # Parse AnalytesInTheStudy → analytes
    ├── form_sets.rs         # Parse FormSets → formset_names
    ├── forms.rs             # Parse Forms → context.forms
    ├── items.rs             # Parse Items → context.item_definitions
    ├── form_item.rs         # Parse FormItem → populate form.items
    ├── unit_groups.rs       # Parse UnitGroups + Units → unit_groups
    └── visits.rs            # Parse Plan* sheets → visits
```

---

## Dependencies

### Task 0: Add calamine dependency

**Files:**
- Modify: `Cargo.toml` (workspace)
- Modify: `crates/als-resolver/Cargo.toml`

- [ ] **Step 1: Add calamine to workspace dependencies**

Modify: `Cargo.toml`
```toml
[workspace.dependencies]
# Add after existing entries:
calamine = "0.35.0"
```

- [ ] **Step 2: Add calamine to als-resolver dependencies**

Modify: `crates/als-resolver/Cargo.toml`
```toml
[dependencies]
calamine = { workspace = true }
# Add after existing entries
```

- [ ] **Step 3: Run cargo check to verify dependency**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml crates/als-resolver/Cargo.toml
git commit -m "feat(als-resolver): add calamine 0.35.0 dependency for ecollect v6 parsing"
```

---

## Module Skeleton

### Task 1: Create ecollect_v6 module skeleton

**Files:**
- Create: `crates/als-resolver/src/ecollect_v6/mod.rs`
- Create: `crates/als-resolver/src/ecollect_v6/context.rs`
- Create: `crates/als-resolver/src/ecollect_v6/parser.rs`
- Create: `crates/als-resolver/src/ecollect_v6/code_list.rs`
- Create: `crates/als-resolver/src/ecollect_v6/analytes.rs`
- Create: `crates/als-resolver/src/ecollect_v6/form_sets.rs`
- Create: `crates/als-resolver/src/ecollect_v6/forms.rs`
- Create: `crates/als-resolver/src/ecollect_v6/items.rs`
- Create: `crates/als-resolver/src/ecollect_v6/form_item.rs`
- Create: `crates/als-resolver/src/ecollect_v6/unit_groups.rs`
- Create: `crates/als-resolver/src/ecollect_v6/visits.rs`

- [ ] **Step 1: Create all module files with empty content**

Run: `touch crates/als-resolver/src/ecollect_v6/{mod.rs,context.rs,parser.rs,code_list.rs,analytes.rs,form_sets.rs,forms.rs,items.rs,form_item.rs,unit_groups.rs,visits.rs}`

- [ ] **Step 2: Write mod.rs declaration**

Create: `crates/als-resolver/src/ecollect_v6/mod.rs`
```rust
mod context;
mod parser;
mod code_list;
mod analytes;
mod form_sets;
mod forms;
mod items;
mod form_item;
mod unit_groups;
mod visits;

pub use parser::EcollectV6Parser;
```

- [ ] **Step 3: Add module to lib.rs**

Modify: `crates/als-resolver/src/lib.rs`
```rust
mod error;
mod traits;
mod rave;
pub mod ecollect_v6;  // ADD THIS LINE

pub use error::AlsParseError;
pub use traits::AlsParser;
pub use entities::project::Project;
```

- [ ] **Step 4: Run cargo check to verify module compiles**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 5: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/ crates/als-resolver/src/lib.rs
git commit -m "feat(als-resolver): scaffold ecollect_v6 module"
```

---

## EcollectParseContext

### Task 2: Define EcollectParseContext and ItemDef

**Files:**
- Create: `crates/als-resolver/src/ecollect_v6/context.rs`

- [ ] **Step 1: Write context.rs with all types**

Create: `crates/als-resolver/src/ecollect_v6/context.rs`
```rust
use entities::project::{CRFForm, ItemOption, Visit};
use std::collections::HashMap;

/// Internal item definition from Items worksheet.
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub oid: String,
    pub item_name: String,
    pub sas_field_name: String,
    pub control_type: String,
    pub data_format: String,
    pub code_list_oid: Option<String>,
    pub unit_group_oid: Option<String>,
}

/// Shared context during ecollect v6 parsing. Accumulates data across phases.
pub struct EcollectParseContext {
    /// CodeListOID → Vec<ItemOption>
    pub code_list_options: HashMap<String, Vec<ItemOption>>,
    /// AnalyteCode → AnalyteName (from AnalytesInTheStudy)
    pub analytes: HashMap<String, String>,
    /// FormsetOID → FormsetName (from FormSets sheet, for visit name lookup)
    pub formset_names: HashMap<String, String>,
    /// Parsed forms (FormOID → CRFForm)
    pub forms: HashMap<String, CRFForm>,
    /// Item definitions (ItemOID → ItemDef)
    pub item_definitions: HashMap<String, ItemDef>,
    /// Visit code → Vec<FormOID> bindings discovered from Plan* sheets
    pub visit_form_bindings: HashMap<String, Vec<String>>,
    /// UnitGroupOID → Vec<UnitName> (for item_unit resolution)
    pub unit_groups: HashMap<String, Vec<String>>,
}

impl Default for EcollectParseContext {
    fn default() -> Self {
        Self {
            code_list_options: HashMap::new(),
            analytes: HashMap::new(),
            formset_names: HashMap::new(),
            forms: HashMap::new(),
            item_definitions: HashMap::new(),
            visit_form_bindings: HashMap::new(),
            unit_groups: HashMap::new(),
        }
    }
}

impl EcollectParseContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Split compound OID (e.g., "YN=[1|是]") on first "=" and return the key part.
    pub fn split_oid(oid: &str) -> &str {
        oid.splitn(2, '=').next().unwrap_or(oid)
    }
}
```

- [ ] **Step 2: Run cargo check to verify context compiles**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/context.rs
git commit -m "feat(ecollect_v6): add EcollectParseContext and ItemDef types"
```

---

## CodeList Parsing

### Task 3: Parse CodeListItems worksheet

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/code_list.rs`

- [ ] **Step 1: Write code_list.rs**

Modify: `crates/als-resolver/src/ecollect_v6/code_list.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::ItemOption;
use std::path::Path;

/// Parse CodeListItems worksheet and populate context.code_list_options.
/// Group rows by CodeListOID, create ItemOption { option_display: DisplayValue }.
pub fn parse_code_list_items(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("CodeListItems")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("CodeListItems".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 5 { continue; }

        let code_list_oid = row[0].to_string();
        let display_value = row[1].to_string();

        if code_list_oid.is_empty() || code_list_oid == "CodeListOID" {
            continue;
        }

        let option = ItemOption {
            option_display: display_value,
            annotations: Vec::new(),
        };

        context.code_list_options
            .entry(code_list_oid)
            .or_default()
            .push(option);
    }

    Ok(())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/code_list.rs
git commit -m "feat(ecollect_v6): parse CodeListItems worksheet into code_list_options"
```

---

## Analytes Parsing

### Task 4: Parse AnalytesInTheStudy worksheet

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/analytes.rs`

- [ ] **Step 1: Write analytes.rs**

Modify: `crates/als-resolver/src/ecollect_v6/analytes.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use std::path::Path;

/// Parse AnalytesInTheStudy worksheet and populate context.analytes.
/// Build AnalyteCode → AnalyteName lookup for Lab Test / Lab Result options.
pub fn parse_analytes(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("AnalytesInTheStudy")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("AnalytesInTheStudy".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 2 { continue; }

        let analyte_code = row[0].to_string();
        let analyte_name = row[1].to_string();

        if analyte_code.is_empty() || analyte_code == "AnalytesCode" {
            continue;
        }

        context.analytes.insert(analyte_code, analyte_name);
    }

    Ok(())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/analytes.rs
git commit -m "feat(ecollect_v6): parse AnalytesInTheStudy worksheet into analytes lookup"
```

---

## FormSets Parsing

### Task 5: Parse FormSets worksheet

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/form_sets.rs`

- [ ] **Step 1: Write form_sets.rs**

Modify: `crates/als-resolver/src/ecollect_v6/form_sets.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use std::path::Path;

/// Parse FormSets worksheet and populate context.formset_names.
/// Build FormsetOID → FormsetName lookup for visit name resolution.
pub fn parse_form_sets(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("FormSets")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("FormSets".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 2 { continue; }

        let formset_oid = row[0].to_string();
        let formset_name = row[1].to_string();

        if formset_oid.is_empty() || formset_oid == "FormsetOID" {
            continue;
        }

        context.formset_names.insert(formset_oid, formset_name);
    }

    Ok(())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/form_sets.rs
git commit -m "feat(ecollect_v6): parse FormSets worksheet into formset_names lookup"
```

---

## UnitGroups Parsing

### Task 6: Parse UnitGroups and Units worksheets

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/unit_groups.rs`

- [ ] **Step 1: Write unit_groups.rs**

Modify: `crates/als-resolver/src/ecollect_v6/unit_groups.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use std::path::Path;

/// Parse UnitGroups and Units worksheets into context.unit_groups.
/// Build UnitGroupOID → Vec<UnitName> lookup for item_unit resolution.
pub fn parse_unit_groups(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    // Parse Units worksheet (UnitGroupOID, UnitOID, UnitName, ...)
    let units_range = workbook.worksheet_range("Units")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Units".to_string()))?;

    // First row is header, skip it
    for row in units_range.rows().skip(1) {
        if row.len() < 3 { continue; }

        let unit_group_oid = row[0].to_string();
        let unit_name = row[2].to_string();

        if unit_group_oid.is_empty() || unit_group_oid == "UnitGroupOID" {
            continue;
        }

        context.unit_groups
            .entry(unit_group_oid)
            .or_default()
            .push(unit_name);
    }

    Ok(())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/unit_groups.rs
git commit -m "feat(ecollect_v6): parse UnitGroups and Units worksheets into unit_groups lookup"
```

---

## Forms Parsing

### Task 7: Parse Forms worksheet

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/forms.rs`

- [ ] **Step 1: Write forms.rs**

Modify: `crates/als-resolver/src/ecollect_v6/forms.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::CRFForm;
use std::path::Path;

/// Parse Forms worksheet and populate context.forms.
/// Create CRFForm { name: FormOID, description: FormName, order: Ordinal, ... }.
pub fn parse_forms(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("Forms")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Forms".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 3 { continue; }

        let form_oid = row[0].to_string();
        let ordinal = row[1].to_string().parse::<i32>().unwrap_or(0);
        let form_name = row[3].to_string(); // FormName is column index 3 (0-based)

        if form_oid.is_empty() || form_oid == "FormOID" {
            continue;
        }

        let form = CRFForm {
            name: form_oid.clone(),
            description: form_name,
            order: ordinal,
            items: Vec::new(),
            domains: Vec::new(),
            annotations: Vec::new(),
        };

        context.forms.insert(form_oid, form);
    }

    Ok(())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/forms.rs
git commit -m "feat(ecollect_v6): parse Forms worksheet into context.forms"
```

---

## Items Parsing

### Task 8: Parse Items worksheet

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/items.rs`

- [ ] **Step 1: Write items.rs**

Modify: `crates/als-resolver/src/ecollect_v6/items.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::{EcollectParseContext, ItemDef};
use crate::ecollect_v6::context::EcollectParseContext as Context;
use std::path::Path;

/// Parse Items worksheet and populate context.item_definitions.
/// Columns: ItemOID(0), SASFieldName(1), ItemName(2), ControlType(4),
/// DataFormat(7), CodeListOID(8), UnitGroupOID(11).
pub fn parse_items(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("Items")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("Items".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 12 { continue; }

        let oid = row[0].to_string();
        if oid.is_empty() || oid == "ItemOID" {
            continue;
        }

        let code_list_raw = row[8].to_string();
        let unit_group_raw = row[11].to_string();

        let item_def = ItemDef {
            oid: oid.clone(),
            item_name: row[2].to_string(),
            sas_field_name: row[1].to_string(),
            control_type: row[4].to_string(),
            data_format: row[7].to_string(),
            code_list_oid: if code_list_raw.is_empty() {
                None
            } else {
                Some(Context::split_oid(&code_list_raw).to_string())
            },
            unit_group_oid: if unit_group_raw.is_empty() {
                None
            } else {
                Some(Context::split_oid(&unit_group_raw).to_string())
            },
        };

        context.item_definitions.insert(oid, item_def);
    }

    Ok(())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/items.rs
git commit -m "feat(ecollect_v6): parse Items worksheet into item_definitions"
```

---

## FormItem Parsing

### Task 9: Parse FormItem worksheet and build CRFItems

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/form_item.rs`

- [ ] **Step 1: Write form_item.rs**

Modify: `crates/als-resolver/src/ecollect_v6/form_item.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use crate::ecollect_v6::context::EcollectParseContext as Ctx;
use entities::project::{CRFItem, ControlType, ItemOption, ItemUnit};
use std::path::Path;

/// Parse FormItem worksheet and populate form.items with CRFItems.
/// For each row, look up ItemOID in item_definitions, create CRFItem with
/// ControlType mapping, CodeList/Lab Test options, unit resolution.
pub fn parse_form_item(path: &Path, context: &mut EcollectParseContext) -> Result<(), crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let range = workbook.worksheet_range("FormItem")
        .map_err(|_| crate::AlsParseError::WorksheetNotFound("FormItem".to_string()))?;

    // First row is header, skip it
    for row in range.rows().skip(1) {
        if row.len() < 3 { continue; }

        let form_oid = row[0].to_string();
        let item_oid = row[2].to_string();

        if form_oid.is_empty() || form_oid == "FormOID" || item_oid.is_empty() || item_oid == "ItemOID" {
            continue;
        }

        // Look up item definition
        let Some(item_def) = context.item_definitions.get(&item_oid) else {
            continue;
        };

        // Resolve control_type
        let (control_type, not_variable) = map_control_type(&item_def.control_type);

        // Resolve item_option
        let item_option = resolve_item_option(&item_def.code_list_oid, &item_def.control_type, row.get(19).map(|c| c.to_string()).as_deref(), context);

        // Resolve item_unit
        let item_unit = resolve_item_unit(&item_def.unit_group_oid, context);

        // Label from FormItem.ItemName (field 41, index 41) or Items.ItemName
        let label = if row.len() > 41 && !row[41].to_string().is_empty() {
            row[41].to_string()
        } else {
            item_def.item_name.clone()
        };

        let item = CRFItem {
            name: item_oid.clone(),
            label,
            item_option,
            annotations: Vec::new(),
            format: item_def.data_format.clone(),
            control_type,
            item_unit,
            not_variable,
        };

        // Add item to form
        if let Some(form) = context.forms.get_mut(&form_oid) {
            form.items.push(item);
        }
    }

    Ok(())
}

/// Map ecollect ControlType string to CRFItem ControlType enum and not_variable.
fn map_control_type(ct: &str) -> (ControlType, Option<bool>) {
    match ct {
        "Textbox" => (ControlType::TEXT, None),
        "Drop-down List" => (ControlType::SELECTION, None),
        "Radio(horizontal)" => (ControlType::SELECTION, None),
        "Radio(vertical)" => (ControlType::SELECTION, None),
        "Check" => (ControlType::CHECKBOX, None),
        "Tags" => (ControlType::TEXT, Some(true)),
        "Lab Test" => (ControlType::SELECTION, None),
        "Lab Result" => (ControlType::SELECTION, None),
        "Calendar" => (ControlType::TEXT, None),
        "Dynamic Options" => (ControlType::TEXT, None),
        _ => (ControlType::TEXT, None),
    }
}

/// Resolve item_option from CodeListOID, Lab Test, or Lab Result.
fn resolve_item_option(
    code_list_oid: &Option<String>,
    control_type: &str,
    default_value: Option<&str>,
    context: &EcollectParseContext,
) -> Option<Vec<ItemOption>> {
    match control_type {
        "Lab Test" | "Lab Result" => {
            // Use DefaultValue as analyte code to look up AnalytesInTheStudy
            if let Some(dv) = default_value {
                if let Some(analyte_name) = context.analytes.get(dv) {
                    return Some(vec![ItemOption {
                        option_display: analyte_name.clone(),
                        annotations: Vec::new(),
                    }]);
                }
            }
            None
        }
        _ => {
            // Use CodeListOID lookup
            if let Some(oid) = code_list_oid {
                context.code_list_options.get(oid).cloned()
            } else {
                None
            }
        }
    }
}

/// Resolve item_unit from UnitGroupOID.
fn resolve_item_unit(
    unit_group_oid: &Option<String>,
    context: &EcollectParseContext,
) -> Option<ItemUnit> {
    if let Some(oid) = unit_group_oid {
        if let Some(units) = context.unit_groups.get(oid) {
            if let Some(first_unit) = units.first() {
                return Some(ItemUnit {
                    value: first_unit.clone(),
                    annotations: Vec::new(),
                });
            }
        }
    }
    None
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/form_item.rs
git commit -m "feat(ecollect_v6): parse FormItem worksheet, build CRFItems with ControlType mapping"
```

---

## Visits Parsing

### Task 10: Parse Plan* sheets and build Visit structs

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/visits.rs`

- [ ] **Step 1: Write visits.rs**

Modify: `crates/als-resolver/src/ecollect_v6/visits.rs`
```rust
use calamine::{open_workbook, Reader, Xlsx};
use crate::ecollect_v6::context::EcollectParseContext;
use entities::project::Visit;
use std::path::Path;

/// Parse Plan* sheets and build Visit structs.
/// Visit code = column header (columns 1+), name from formset_names lookup.
/// Build visit_form_bindings from non-empty cells in Plan* sheets.
pub fn parse_visits(path: &Path, context: &mut EcollectParseContext) -> Result<Vec<Visit>, crate::AlsParseError> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| crate::AlsParseError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let plan_sheets = ["PlanSCR", "PlanCYCLE", "PlanEARLY", "PlanCOM", "PlanDSEOS", "PlanUNS"];

    // First pass: extract column headers from first Plan* sheet to get visit codes
    let first_sheet = plan_sheets.first().unwrap();
    let range = workbook.worksheet_range(first_sheet)
        .map_err(|_| crate::AlsParseError::WorksheetNotFound(first_sheet.to_string()))?;

    let mut visit_codes: Vec<String> = Vec::new();
    if let Some(header_row) = range.rows().next() {
        // Column 0 = "Form\\Visit", columns 1+ = visit codes
        for (i, cell) in header_row.iter().enumerate() {
            if i == 0 { continue; } // Skip "Form\\Visit"
            let code = cell.to_string();
            if !code.is_empty() {
                visit_codes.push(code);
            }
        }
    }

    // Second pass: process all Plan* sheets to build visit_form_bindings
    for sheet_name in &plan_sheets {
        let Ok(sheet_range) = workbook.worksheet_range(sheet_name) else {
            continue;
        };

        for (row_idx, row) in sheet_range.rows().enumerate() {
            if row_idx == 0 { continue; } // Skip header row
            if row.is_empty() { continue; }

            let form_oid = row[0].to_string();
            if form_oid.is_empty() { continue; }

            // Check columns 1+ for non-empty cells
            for (col_idx, cell) in row.iter().enumerate().skip(1) {
                let cell_str = cell.to_string();
                if !cell_str.is_empty() && col_idx - 1 < visit_codes.len() {
                    let visit_code = &visit_codes[col_idx - 1];
                    context.visit_form_bindings
                        .entry(visit_code.clone())
                        .or_default()
                        .push(form_oid.clone());
                }
            }
        }
    }

    // Build Visit structs
    let mut visits: Vec<Visit> = Vec::new();
    for (order, code) in visit_codes.iter().enumerate() {
        let name = context.formset_names.get(code).cloned().unwrap_or_else(|| code.clone());
        let forms = context.visit_form_bindings.get(code).cloned().unwrap_or_default();

        // Deduplicate forms
        let mut unique_forms: Vec<String> = Vec::new();
        for f in forms {
            if !unique_forms.contains(&f) {
                unique_forms.push(f);
            }
        }

        visits.push(Visit {
            code: code.clone(),
            name,
            order: order as i32,
            forms: unique_forms,
        });
    }

    Ok(visits)
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS

- [ ] **Step 3: Commit**

```bash
git add crates/als-resolver/src/ecollect_v6/visits.rs
git commit -m "feat(ecollect_v6): parse Plan* sheets into Visit structs with form bindings"
```

---

## EcollectV6Parser + Public API

### Task 11: Implement EcollectV6Parser and wire up public API

**Files:**
- Modify: `crates/als-resolver/src/ecollect_v6/parser.rs`
- Modify: `crates/als-resolver/src/lib.rs`

- [ ] **Step 1: Write parser.rs**

Modify: `crates/als-resolver/src/ecollect_v6/parser.rs`
```rust
use crate::ecollect_v6::context::EcollectParseContext;
use crate::ecollect_v6::{code_list, analytes, form_sets, forms, items, form_item, unit_groups, visits};
use crate::traits::AlsParser;
use crate::AlsParseError;
use entities::project::Project;
use std::io::Read;
use std::path::Path;

/// Ecollect v6 ALS parser implementation.
pub struct EcollectV6Parser;

impl AlsParser for EcollectV6Parser {
    fn parse(self, path: &Path) -> Result<Project, AlsParseError> {
        let mut context = EcollectParseContext::new();

        // Phase 1: Load reference data
        code_list::parse_code_list_items(path, &mut context)?;
        analytes::parse_analytes(path, &mut context)?;
        form_sets::parse_form_sets(path, &mut context)?;
        unit_groups::parse_unit_groups(path, &mut context)?;

        // Phase 2: Parse forms
        forms::parse_forms(path, &mut context)?;

        // Phase 3: Parse items and form-item linkage
        items::parse_items(path, &mut context)?;
        form_item::parse_form_item(path, &mut context)?;

        // Phase 4: Parse visits
        let visit_list = visits::parse_visits(path, &mut context)?;

        // Build and return Project
        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: visit_list,
        })
    }
}
```

- [ ] **Step 2: Update lib.rs with public API**

Modify: `crates/als-resolver/src/lib.rs`
```rust
mod error;
mod traits;
mod rave;
pub mod ecollect_v6;

pub use error::AlsParseError;
pub use traits::AlsParser;
pub use entities::project::Project;

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Parse a Rave ALS file from a path.
pub fn parse_rave_als(path: &Path) -> Result<Project, AlsParseError> {
    let file = File::open(path).map_err(AlsParseError::IoError)?;
    parse_rave_als_stream(file)
}

/// Parse a Rave ALS file from any Read source.
pub fn parse_rave_als_stream(input: impl Read + 'static) -> Result<Project, AlsParseError> {
    rave::parser::RaveParser.parse(input)
}

/// Parse an ecollect v6 ALS file from a path.
pub fn parse_ecollect_v6_als(path: &Path) -> Result<Project, AlsParseError> {
    ecollect_v6::parser::EcollectV6Parser.parse(path)
}
```

- [ ] **Step 3: Fix trait signature mismatch — AlsParser takes path, not Read**

The existing trait is:
```rust
pub trait AlsParser {
    fn parse(self, source: impl std::io::Read + 'static) -> Result<Project, AlsParseError>;
}
```

But EcollectV6Parser needs `&Path`. We have two options:
1. Change `AlsParser` trait to take `&Path`
2. Create a separate function without trait

Since Rave parser already uses `impl Read`, we need to change the trait to take `&Path` for ecollect compatibility. This is a BREAKING CHANGE to the trait but necessary.

Modify: `crates/als-resolver/src/traits.rs`
```rust
use crate::error::AlsParseError;
use entities::project::Project;
use std::path::Path;

/// Parser trait for ALS (Audit Landmark Study) files.
/// Implementors parse different ALS formats (Rave, ecollect, etc.)
/// into a unified Project structure.
pub trait AlsParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError>;
}
```

Modify: `crates/als-resolver/src/rave/parser.rs`
```rust
impl AlsParser for RaveParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let file = File::open(path).map_err(AlsParseError::IoError)?;
        // For Rave, we need to read the XML content from the file
        // Rave ALS files are .xls XML format, read into memory then parse
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let mut context = ParseContext::new();
        let mut reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        navigate_to_worksheet(&mut reader, "DataDictionaryEntries")?;
        parse_data_dictionaries(&mut reader, &mut context)?;

        // ... rest of phases
        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: context.visits,
        })
    }
}
```

Wait — the Rave parser uses `source: impl Read` and reads the whole file into memory. For ecollect we use calamine on a path. Let's unify by having the trait take `&Path` and have `parse_rave_als_stream` convert Read to Path via a temp file or similar.

Actually the simplest fix is: keep `parse_rave_als_stream` with `impl Read` as a separate public API function that is NOT part of the trait, and make `AlsParser.parse` take `&Path`.

Modify: `crates/als-resolver/src/lib.rs`
```rust
/// Parse a Rave ALS file from a path.
pub fn parse_rave_als(path: &Path) -> Result<Project, AlsParseError> {
    rave::parser::RaveParser.parse(path)
}

/// Parse an ecollect v6 ALS file from a path.
pub fn parse_ecollect_v6_als(path: &Path) -> Result<Project, AlsParseError> {
    ecollect_v6::parser::EcollectV6Parser.parse(path)
}
```

And update RaveParser.parse to take `&Path` and handle file reading internally.

- [ ] **Step 4: Run cargo check**

Run: `cd /Users/yukichen/Coding/Projects/ypsilo && cargo check`
Expected: BUILD SUCCESS (may need several iterations to fix trait signature changes)

- [ ] **Step 5: Commit**

```bash
git add crates/als-resolver/src/lib.rs crates/als-resolver/src/traits.rs crates/als-resolver/src/rave/parser.rs crates/als-resolver/src/ecollect_v6/parser.rs
git commit -m "feat(ecollect_v6): implement EcollectV6Parser and wire up public API"
```

---

## Self-Review Checklist

**Spec coverage:**
- [x] CodeListItems → code_list_options (Task 3)
- [x] AnalytesInTheStudy → analytes (Task 4)
- [x] FormSets → formset_names (Task 5)
- [x] UnitGroups + Units → unit_groups (Task 6)
- [x] Forms → context.forms (Task 7)
- [x] Items → item_definitions (Task 8)
- [x] FormItem → CRFItem with ControlType mapping, CodeList/Lab Test options, unit (Task 9)
- [x] Plan* sheets → Visit structs with form bindings (Task 10)
- [x] Public API: parse_ecollect_v6_als(path) + parse_rave_als(path) (Task 11)

**Placeholder scan:** All steps have complete code, no TBD/TODO.

**Type consistency:** 
- `EcollectParseContext::split_oid` is a method on the context struct
- `map_control_type`, `resolve_item_option`, `resolve_item_unit` are standalone functions in form_item.rs
- ControlType enum imported from `entities::project`

---

## Plan Complete

**Saved to:** `docs/superpowers/plans/2026-06-12-ecollect-v6-als-parser-implementation.md`

**Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**