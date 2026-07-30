# Crate Terminology

## Scope

1. Define the Terminology Data Model
2. Deserialize the SDTM or ADaM Terminology data from xls format 

## Data Model

Data model will be describe with rust struct

```rust
struct TerminologyVersion {
    name: String,
    codelist: Vec<CodeList>
}

struct CodeList {
    code: String,
    extensible: bool,
    name: String,
    submission_value: String,
    synonym: String,
    definition: String,
    nci_preferred_term: String,
    code_list: Vec<CodeItem>,
}

struct CodeItem {
    code: String,
    submission_value: String,
    synonym: String,
    definition: String,
    nci_preferred_term: String,
}
```

## How to Deserialize the XLS File

### Contents:

Row 0 is the title, skip it, rest of rows are code list or code item;

Columns:

- 0: Code: map to field `code` in `CodeList` or `CodeItem`
- 1: Codelist Code: If empty, means this row is a code list, else code item
- 2: Codelist Extensible (Yes/No): map to field `extensible` in `CodeList`
- 3: Codelist Name: map to field `name` in `CodeList`
- 4: CDISC Submission Value: map to field `submission_value` in `CodeList` or `CodeItem`
- 5: CDISC Synonym(s): map to field `synonym` in `CodeList` or `CodeItem`
- 6: CDISC Definition: map to field `definition` in `CodeList` or `CodeItem`
- 7: NCI Preferred Term: map to field `nci_preferred_term` in `CodeList` or `CodeItem`


1. Find the target sheet name pattern: `XXXX Terminology yyyy-mm-dd`, then read it
2. extract the yyyy-mm-dd information, use as the field `name` in struct `TerminologyVersion`, and init a TerminologyVersion struct
3. Reconstruct the records into CodeList, and save to the field `codelist` in struct `TerminologyVersion`
4. return the TerminologyVersion struct

## Attentions

1. use `thiserror` crate in workspace to wrap errors
2. use crate calamine(0.36.1) to read xls files
3. export the data model and the deserialize function from the crate

## Test

You can use following xls files for tests:

- SDTM Terminology: @.mock_data/terminologies/ADaM Terminology.xls
- ADaM Terminology: @.mock_data/terminologies/SDTM Terminology.xls

