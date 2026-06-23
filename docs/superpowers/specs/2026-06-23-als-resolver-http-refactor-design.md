# ALS Resolver HTTP Refactor Design

**Date:** 2026-06-23
**Status:** Approved

## Goal

Refactor `parse_xxx_als` functions in `crates/als-resolver/src/lib.rs` to accept `impl Read` sources, enabling HTTP/serverless usage without local file storage.

## Approach

Extend the `AlsParser` trait with a `parse_reader` method. Existing path-based functions become thin wrappers.

## API Changes

### AlsParser Trait

```rust
pub trait AlsParser {
    /// Parse from a file path (existing API, preserved for backward compatibility).
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let file = File::open(path)?;
        self.parse_reader(BufReader::new(file))
    }

    /// Parse from any `impl Read` source.
    fn parse_reader(&self, reader: impl Read) -> Result<Project, AlsParseError>;
}
```

### New Public Functions (lib.rs)

```rust
/// Parse a Rave ALS file from any `impl Read` source.
pub fn parse_rave_als_from(reader: impl Read) -> Result<Project, AlsParseError> {
    RaveParser.parse_reader(reader)
}

/// Parse an ecollect v6 ALS file from any `impl Read` source.
pub fn parse_ecollect_v6_als_from(reader: impl Read) -> Result<Project, AlsParseError> {
    EcollectV6Parser.parse_reader(reader)
}

/// Parse an ecollect legacy ALS file from any `impl Read` source.
pub fn parse_ecollect_legacy_als_from(reader: impl Read) -> Result<Project, AlsParseError> {
    EcollectLegacyParser.parse_reader(reader)
}
```

## Parser Implementations

### Rave

**Current:** `quick-xml::Reader::from_reader` already accepts `impl Read`. The `parse(path)` reads file into `Vec<u8>` then creates a new reader per phase.

**Change:** Move the multi-phase logic to `parse_reader(impl Read)`. The caller passes a reader, and we create a new `quick-xml::Reader` from it per phase (same as current, but without file opening).

**Data flow:**
```
impl Read
  → quick_xml::Reader::from_reader()
  → navigate_to_worksheet()
  → parse_data_dictionaries() / parse_forms() / etc.
  → Project
```

### Ecollect v6

**Current:** Uses `calamine::open_workbook(path)` which opens a file by path.

**Change:** Use `calamine::open_workbook_from_rs(reader)` which accepts `impl Read`. Wrap in `BufReader` for efficient buffering.

**Data flow:**
```
impl Read
  → BufReader::new(reader)
  → calamine::open_workbook_from_rs::<_, Xlsx>(BufReader::new(reader))?
  → worksheet_range()
  → Project
```

### Ecollect Legacy

**Same as Ecollect v6** — use `calamine::open_workbook_from_rs(reader)`.

## Changes Summary

| File | Change |
|------|--------|
| `src/traits.rs` | Add `parse_reader` method to `AlsParser` trait |
| `src/rave/parser.rs` | Implement `parse_reader(impl Read)`; `parse(path)` becomes thin wrapper |
| `src/ecollect_v6/parser.rs` | Implement `parse_reader(impl Read)`; change `open_workbook(path)` to `open_workbook_from_rs` |
| `src/ecollect_legacy/parser.rs` | Implement `parse_reader(impl Read)`; change `open_workbook(path)` to `open_workbook_from_rs` |
| `src/lib.rs` | Add `parse_xxx_als_from` functions; add `BufReader` import |

## Error Handling

No new error types needed. `AlsParseError` covers `IoError`, `XmlError`, `WorksheetNotFound`.

## Testing

- Existing path-based tests continue to work (wrapper delegates to new implementation)
- Add new tests using `Cursor<Vec<u8>>` as reader source for each parser

## HTTP Usage Example

```rust
// Example: Axum handler
async fn upload_als(mut body: Body) -> Result<Json<Project>, StatusCode> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    let project = parse_rave_als_from(bytes.reader())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    Ok(Json(project))
}
```
