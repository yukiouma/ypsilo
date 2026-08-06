# terminology

Deserialises a CDISC SDTM or ADaM terminology workbook (`.xls`/`.xlsx`) into a
typed `TerminologyVersion` containing all the `CodeList`s and their `CodeItem`s.

## Usage

```rust
use terminology::{from_path, TerminologyVersion};

let version: TerminologyVersion = from_path("path/to/SDTM Terminology.xls")?;
println!("{} codelists dated {}", version.codelist.len(), version.name);

let first = &version.codelist[0];
println!("{} ({}): {} items", first.name, first.code, first.code_list.len());
```

In-memory byte slices and arbitrary `Read + Seek` readers are supported via
`from_bytes` and `from_reader` respectively.

## Error handling

Every entry point returns `Result<TerminologyVersion, TerminologyError>`. The
error variants cover I/O failures, malformed workbooks, missing or ambiguous
sheet names, unparseable `Extensible` values, orphan code items, and any cell
type the workbook should not contain. Each row-level variant carries the sheet
name and 1-indexed row number for easy debugging.

## Data model

See the design spec at
[`docs/superpowers/specs/2026-07-30-terminology-crate-design.md`](../../docs/superpowers/specs/2026-07-30-terminology-crate-design.md)
for the full type definitions.
