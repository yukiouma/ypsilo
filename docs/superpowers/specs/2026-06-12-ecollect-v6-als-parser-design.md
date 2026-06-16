# EcollectV6 ALS Parser — Design Specification

**Date:** 2026/06/12
**Author:** Claude
**Status:** Draft

---

## Overview

Implement an `AlsParser` for ecollect v6 ALS file format in `crates/als-resolver`. The parser reads an Excel (.xlsx) file and produces a `Project` entity — the same output as the existing Rave parser. Sheets not representable in `Project` (Checks, Derivations, LabVariableMappings, etc.) are ignored.

---

## Architecture

### Module Structure

```
crates/als-resolver/src/
├── lib.rs              # Public API: parse_ecollect_v6_als(path/stream)
├── error.rs            # AlsParseError (existing)
├── traits.rs           # AlsParser trait (existing)
├── rave.rs             # RaveParser + submodules (existing)
└── ecollect_v6.rs      # EcollectV6Parser + submodules (NEW)
    ├── parser.rs       # EcollectV6Parser struct + impl AlsParser
    ├── context.rs      # EcollectParseContext (shared lookup tables)
    ├── forms.rs        # Parse Forms worksheet
    ├── items.rs        # Parse Items worksheet
    ├── form_item.rs    # Parse FormItem worksheet (form-item linkage)
    ├── code_list.rs    # Parse CodeList + CodeListItems (item options)
    ├── analytes.rs     # Parse AnalytesInTheStudy (Lab Test options)
    └── visits.rs       # Parse Plan* sheets → Visit structs
```

### Key Design Decisions

1. **Output alignment with Rave** — Produce `Project` with `CRFForm`, `CRFItem`, `Visit` only. Extra sheets (Checks, Derivations, etc.) are ignored.
2. **calamine 0.35.0** — Excel parsing via `calamine` crate (Office Open XML format).
3. **Lazy parsing** — Each worksheet is parsed on-demand in sequence; no full-file load into memory upfront.
4. **Fail fast** — Return error on first parsing problem, no partial results.

---

## Public API

```rust
// lib.rs
pub fn parse_ecollect_v6_als(path: &Path) -> Result<Project, AlsParseError> {
    let file = File::open(path).map_err(AlsParseError::IoError)?;
    parse_ecollect_v6_als_stream(file)
}

pub fn parse_ecollect_v6_als_stream(input: impl Read + 'static) -> Result<Project, AlsParseError> {
    ecollect_v6::parser::EcollectV6Parser.parse(input)
}
```

**Returns:** `Project` containing:
- `forms: Vec<CRFForm>` — parsed forms with items
- `visit: Vec<Visit>` — parsed visits with form bindings from Plan* sheets

---

## Parsing Flow

### Phase 1: Load Reference Data (prerequisites for items)

1. Navigate to **CodeListItems** worksheet
   - Parse rows into lookup: `CodeListOID → Vec<ItemOption>`
2. Navigate to **AnalytesInTheStudy** worksheet
   - Parse rows into lookup: `AnalyteCode → AnalyteName` (for Lab Test control type)

### Phase 2: Parse Forms

1. Navigate to **Forms** worksheet (14 cols × 40 rows)
2. For each row where `FormOID` is non-empty and not the header:
   - Create `CRFForm { name: FormOID, description: FormName, order: Ordinal, items: Vec::new(), domains: Vec::new(), annotations: Vec::new() }`
   - Store in context by OID

### Phase 3: Parse Items + FormItem (linkage)

1. Navigate to **Items** worksheet (14 cols × 273 rows)
   - Parse all rows into `ItemDef { oid, item_name, sas_field_name, control_type, data_format, code_list_oid, unit_group_oid }` stored in `context.item_definitions` (HashMap<ItemOID, ItemDef>)

2. Navigate to **FormItem** worksheet (53 cols × 329 rows)
   - For each row where `FormOID` and `ItemOID` are non-empty:
     - Look up `ItemOID` in `context.item_definitions`
     - Create `CRFItem` from ItemDef + FormItem fields
     - Add to corresponding form's items list

### Phase 4: Parse Visits

1. Navigate to **PlanSCR** worksheet (or any Plan* sheet) to extract column headers as visit names (row 1, columns 1..N)
2. For each Plan* sheet (PlanSCR, PlanCYCLE, PlanEARLY, PlanCOM, PlanDSEOS, PlanUNS):
   - For each row and non-empty cell in columns 2..N, mark that form OID as scheduled for that visit column
3. Build `Vec<Visit>` from all discovered visit columns

---

## CRFItem Mapping

| CRFItem field | Source | Notes |
|---------------|--------|-------|
| `name` | `FormItem.ItemOID` → `Items.ItemOID` | |
| `label` | `FormItem.ItemName` (field 41) or `Items.ItemName` | Prefer FormItem's ItemName |
| `format` | `Items.DataFormat` | NOT DefaultValue |
| `control_type` | `Items.ControlType` → `ControlType` enum | See ControlType mapping below |
| `item_option` | `Items.CodeListOID` → `CodeListItems` | Split by "=" to get OID; Or Lab Test / Lab Result special handling |
| `item_unit` | `Items.UnitGroupOID` → `Units.UnitName` | Split by "=" to get OID; Lookup via UnitGroups worksheet |
| `not_variable` | `true` if `Items.ControlType == "Tags"` | Otherwise `None` |
| `annotations` | `Vec::new()` | Always empty |

### ControlType Mapping

| ecollect Value | CRFItem.control_type | Notes |
|----------------|----------------------|-------|
| "Textbox" | `TEXT` | |
| "Drop-down List" | `SELECTION` | |
| "Radio(horizontal)" | `SELECTION` | |
| "Radio(vertical)" | `SELECTION` | |
| "Check" | `CHECKBOX` | |
| "Tags" | `TEXT` | `not_variable = true` |
| "Lab Test" | `SELECTION` | Options from `AnalytesInTheStudy` via `DefaultValue` |
| "Lab Result" | `SELECTION` | Options from `AnalytesInTheStudy` via `DefaultValue` |
| "Calendar" | `TEXT` | Date/datetime input, stored as TEXT |
| "Dynamic Options" | `TEXT` | Options resolved at runtime, stored as text |
| (other) | `TEXT` | Fallback |

### Lab Test / Lab Result Special Handling

When `Items.ControlType == "Lab Test"` or `"Lab Result"`:
1. Get `FormItem.DefaultValue` (the analyte code)
2. Look up `AnalyteCode` in `AnalytesInTheStudy` lookup → `AnalyteName`
3. Create `ItemOption { option_display: AnalyteName, annotations: Vec::new() }` as sole option

---

## Visit Mapping

| Visit field | Source | Notes |
|-------------|--------|-------|
| `code` | Plan* sheet column header | Column 1 = "Form\\Visit", columns 2+ = visit codes (FormsetOID) |
| `name` | FormSets.FormsetName | Match code against FormSets.FormsetOID → FormsetName |
| `order` | Column index (0-based) | |
| `forms` | Form OIDs with non-empty cell in any Plan* sheet | Deduplicated across all Plan* sheets |

**Visit discovery:** All Plan* sheets share the same column headers. A form is bound to a visit if any Plan* sheet has a non-empty value in that cell. Visits are ordered by column index (column 2 = first visit = order 0, column 3 = order 1, etc.).

**Visit name resolution:** After extracting column headers as visit codes, look up each code in the FormSets sheet (`FormsetOID` → `FormsetName`) to resolve the visit name. If no match found in FormSets, use the code as the name.

---

## CodeList Lookup

### Compound OID Format

Both `CodeListOID` and `UnitGroupOID` in ecollect v6 use a compound format: `OID=[options]`. For example:
- `"YN=[1|是,2|否]"` → CodeListOID = `"YN"`
- `"Age=[years]"` → UnitGroupOID = `"Age"`

**Parsing rule:** Split the value on the first `=` character. The portion before `=` is the lookup key.

1. Navigate to **CodeListItems** worksheet
2. Group rows by `CodeListOID`
3. For each `CodeListItem` row:
   - Create `ItemOption { option_display: DisplayValue, annotations: Vec::new() }`
   - Append to lookup for that `CodeListOID`
4. When `Items.CodeListOID` is set and non-empty:
   - Split by "=" → take first part as `CodeListOID`
   - Look up the OID in the code list lookup to populate `CRFItem.item_option`

---

## Error Handling

- **Fail fast** — first error stops parsing, returns `AlsParseError`
- `AlsParseError` variants (existing):
  - `FileNotFound(String)` — path doesn't exist
  - `IoError(String)` — file read or stream error
  - `WorksheetNotFound(String)` — required sheet missing
  - `MissingRequiredField(String)` — OID, Ordinal, etc. missing
  - `InvalidFieldValue(String)` — malformed data

---

## Dependencies

Add to workspace `Cargo.toml` under `[workspace.dependencies]`:
```toml
calamine = "0.35.0"
```

Add to `crates/als-resolver/Cargo.toml`:
```toml
calamine = { workspace = true }
```

---

## Testing Strategy

1. **Unit tests** per module — parse small Excel snippets or mock data
2. **Integration test** — parse `.mock_data/als/ecollect_v6.xlsx` and verify output structure
3. **Edge cases** — empty CodeList, missing Items, Lab Test without analyte match, invisible items

---

## Files to Create

### New files
- `crates/als-resolver/src/ecollect_v6.rs` — module declaration
- `crates/als-resolver/src/ecollect_v6/parser.rs` — EcollectV6Parser + impl AlsParser
- `crates/als-resolver/src/ecollect_v6/context.rs` — EcollectParseContext
- `crates/als-resolver/src/ecollect_v6/forms.rs` — Forms worksheet parsing
- `crates/als-resolver/src/ecollect_v6/items.rs` — Items worksheet parsing
- `crates/als-resolver/src/ecollect_v6/form_item.rs` — FormItem worksheet parsing
- `crates/als-resolver/src/ecollect_v6/code_list.rs` — CodeList + CodeListItems parsing
- `crates/als-resolver/src/ecollect_v6/analytes.rs` — AnalytesInTheStudy parsing
- `crates/als-resolver/src/ecollect_v6/visits.rs` — Plan* sheets → Visit structs

### Modified files
- `crates/als-resolver/src/lib.rs` — add `pub mod ecollect_v6;` and public API functions
- `crates/als-resolver/Cargo.toml` — add `calamine = { workspace = true }`
- `Cargo.toml` — add `calamine = "0.35.0"` to `[workspace.dependencies]`

---

## Parsing Sequence

```
1. CodeListItems    → context.code_list_options (HashMap<CodeListOID, Vec<ItemOption>>)
2. AnalytesInTheStudy → context.analytes (HashMap<AnalyteCode, AnalyteName>)
3. FormSets         → context.formset_names (HashMap<FormsetOID, FormsetName>)
4. Forms            → context.forms (HashMap<FormOID, CRFForm>)
5. Items            → context.item_definitions (HashMap<ItemOID, ItemDef>)
6. FormItem         → populate forms[].items via item_definitions lookup
7. Plan* sheets     → context.visit_form_bindings (HashMap<VisitCode, Vec<FormOID>>)
   → Build Visit structs (code from column header, name from formset lookup)
```

---

## ItemDef Struct (internal)

```rust
struct ItemDef {
    oid: String,
    item_name: String,
    sas_field_name: String,
    control_type: String,   // Raw string from Excel
    data_format: String,
    code_list_oid: Option<String>,   // Split from compound OID (before the "=")
    unit_group_oid: Option<String>,  // Split from compound OID (before the "=")
}
```

**Note:** When parsing `Items.CodeListOID` and `Items.UnitGroupOID`, split on the first `=` character and store only the portion before `=` as the lookup key.

---

## EcollectParseContext Struct

```rust
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
```

---

## UnitGroup Resolution

1. Parse **UnitGroups** worksheet → list of `UnitGroupOID` values
2. Parse **Units** worksheet → build `UnitGroupOID → Vec<UnitName>` lookup
3. When `Items.UnitGroupOID` is set and non-empty:
   - Split by "=" → take first part as `UnitGroupOID`
   - Look up unit group → get first unit name → create `ItemUnit { value: unit_name, annotations: Vec::new() }`

---

## ControlType Mapping (summary)

| ecollect ControlType | CRFItem.control_type | not_variable |
|---------------------|---------------------|--------------|
| "Textbox" | TEXT | `None` |
| "Drop-down List" | SELECTION | `None` |
| "Radio(horizontal)" | SELECTION | `None` |
| "Radio(vertical)" | SELECTION | `None` |
| "Check" | CHECKBOX | `None` |
| "Tags" | TEXT | `Some(true)` |
| "Lab Test" | SELECTION | `None` |
| "Lab Result" | SELECTION | `None` |
| "Calendar" | TEXT | `None` |
| "Dynamic Options" | TEXT | `None` |
| (other) | TEXT | `None` |