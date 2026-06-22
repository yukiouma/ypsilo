# ecollect_legacy Parser Design

## Overview

Implement an ALS parser for `ecollect_legacy.xlsx` files (EDC clinical trial setup Excel workbooks) in the `als-resolver` crate. The parser follows the existing `AlsParser` trait pattern and produces a `Project` entity.

## Scope

Parse the following sheets (core subset, matching ecollect_v6 scope):

| Sheet | Output Entity |
|-------|---------------|
| Events | `Vec<Visit>` |
| Forms | `Vec<CRFForm>` |
| EventForm | Form-visit linkage |
| GroupItems | `Vec<CRFItem>` |
| CodeList | `Vec<CodeList>` |
| CodeListItems | `Vec<ItemOption>` |
| AnalytesInTheStudy | Option source for CRFItem |

**Excluded sheets** (out of scope for this implementation):
DataStructure, Checks, CheckVariables, CheckActions, Derivations, and all other sheets.

## Architecture

### Module Structure

```
crates/als-resolver/src/ecollect_legacy/
├── parser.rs       # EcollectLegacyParser, implements AlsParser trait
├── context.rs      # LegacyParseContext — shared state across phases
├── events.rs       # Events sheet → Visit
├── forms.rs        # Forms sheet → CRFForm
├── event_form.rs   # EventForm sheet → form-visit linkage
├── group_items.rs  # GroupItems sheet → CRFItem
├── code_list.rs    # CodeList + CodeListItems sheets
└── analytes.rs     # AnalytesInTheStudy sheet
```

### Parsing Phases

```
Phase 1: Load reference data
  ├── CodeList sheet → HashMap<OID, CodeList>
  └── CodeListItems sheet → HashMap<CodeListOID, Vec<ItemOption>>

Phase 2: Load AnalytesInTheStudy
  └── AnalytesInTheStudy sheet → HashMap<AnalytesCode, AnalytesInTheStudy>

Phase 3: Parse forms
  └── Forms sheet → Vec<CRFForm>

Phase 4: Parse items
  └── GroupItems sheet → Vec<CRFItem>
      ├── DisplayMode = AnalytesOption → derive options from AnalytesInTheStudy
      └── DisplayMode = CodeList-based → derive options from CodeList + CodeListItems

Phase 5: Parse visits
  └── Events sheet → Vec<Visit>

Phase 6: Link forms to visits
  └── EventForm sheet → populate Visit.forms
```

### DisplayMode → ControlType Mapping

| GroupItems.DisplayMode | ControlType |
|-----------------------|-------------|
| RadioButton | Radio |
| CheckBox | Checkbox |
| DropDown | Select |
| ComboBox | Select |
| TextField | Text |
| Date | DateTime |
| File | File |
| AnalytesOption | Select (options from AnalytesInTheStudy) |
| (unrecognized) | Text (fallback) |

### AnalytesOption Handling

When `GroupItems.DisplayMode = "AnalytesOption"`:
1. Look up the item's `CodeListOID` in `AnalytesInTheStudy` (keyed by `AnalytesCode`)
2. Each analyte becomes an `ItemOption`:
   - `value`: `AnalytesCode`
   - `label`: `AnalytesName`

### Error Handling

Reuse existing `AlsParseError` from `error.rs`. No new error variants required for this scope.

### Public API

```rust
// crates/als-resolver/src/lib.rs
pub fn parse_ecollect_legacy_als(path: &Path) -> Result<Project, AlsParseError>
```

Wire it into the `AlsParser` registry alongside `parse_ecollect_v6_als`.

## Data Flow Summary

```
Events sheet
    └── Visit entities (OID, Name, SortNumber, EventType, etc.)

EventForm sheet
    └── Links FormOID → EventOID with SortNumber

Forms sheet
    └── CRFForm entities (OID, Name, IsSubjectPage, etc.)

GroupItems sheet
    └── CRFItem entities (OID, Name, ControlType, Required, etc.)
        ├── CodeListOID → lookup CodeList + CodeListItems → ItemOption[]
        └── DisplayMode = AnalytesOption → lookup AnalytesInTheStudy → ItemOption[]
```

## Testing

Add integration tests in `crates/als-resolver/tests/ecollect_legacy_parser_integration.rs`:
- Load the legacy mock data file (create `.mock_data/als/ecollect_legacy.xlsx` if needed)
- Parse and verify key entities are extracted correctly
- Test AnalytesOption item produces correct options

## Dependencies

No new dependencies required — uses existing `calamine` for Excel reading and `thiserror` for error handling, both already in workspace dependencies.
