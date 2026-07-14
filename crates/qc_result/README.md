# qc_result

Parses the HTML output produced by the SAS `PROC COMPARE` step into a strongly-typed
[`QcResult`](src/qc_result.rs) that can be consumed from Rust and serialized to JSON.

## Overview

SAS `PROC COMPARE` writes its findings into one or more `<pre class="batch">` blocks
inside an HTML report. This crate walks each block line-by-line, tracks the current
report section via [`ProcessStage`](src/qc_result.rs), and extracts the rows that
matter (dataset headers, variable attribute deltas, observation counts, value
comparison summaries, etc.) into the [`QcResult`](src/qc_result.rs) aggregate.

The whole pipeline is driven by [`QcResultHtmlParser::parse`](src/qc_result.rs),
which takes any `AsRef<Path>` pointing at the report file.

## Usage

```rust
use qc_result::QcResultHtmlParser;

let parser = QcResultHtmlParser::new();
let result = parser.parse("path/to/compare_report.html")?;

// Serialize the parsed result (all top-level structs derive `Serialize`
// with `#[serde(rename_all = "camelCase")]`).
let json = serde_json::to_string_pretty(&result)?;
```

## Error handling

All fallible operations return the crate-local [`Result`](src/qc_result.rs) alias
(`std::result::Result<T, QcResultError>`). [`QcResultError`](src/qc_result.rs) is
a `thiserror` enum with three variants:

| Variant | Cause |
| --- | --- |
| [`QcResultError::Io`](src/qc_result.rs) | The report file could not be read. Carries the path and the underlying `std::io::Error`. |
| [`QcResultError::Selector`](src/qc_result.rs) | The hard-coded CSS selector (`pre.batch`) failed to parse. Carries the selector and a description. |
| [`QcResultError::Regex`](src/qc_result.rs) | A statically compiled regular expression failed to compile (unreachable for the bundled patterns, which use `.expect` with a descriptive message). |

## Top-level data model

[`QcResult`](src/qc_result.rs) holds every section that the parser knows how to
extract. Each field is `Option<…>` and only populated if the corresponding section
was present in the report:

- `dataset_summary` — the two `Dataset` rows compared (`base` and `compare`).
- `variables_summary` — counts of variables in common and with differing attributes.
- `list_of_common_variables_with_differing_attributes` — per-variable attribute
  deltas, paired `base`/`compare`.
- `comparsion_results_for_observations` — raw observation-level mismatch lines.
- `observation_summary` — `First`/`Last` rows plus the textual log lines emitted
  by `PROC COMPARE` (`Number of Observations …`, etc.).
- `values_comparsion_summary` — the five `Values Comparison Summary` counters.
- `variable_with_unequal_values` — one entry per variable that had any unequal values.
- `values_comparsion_results_for_variables` — per-variable observation-level
  records (`base` vs `compare`) for variables with unequal values.

> **Note:** the typo in `ValuesComparsionSummary` / `comparsion` is preserved on
> purpose so that the serialized JSON keys (`valuesComparsionSummary`,
> `comparsionResultsForObservations`, …) stay aligned with existing consumers.

## Testing

```bash
cargo test -p qc-result
```

The single integration test (`test_qc_result_parse`) writes a representative SAS
`PROC COMPARE` HTML report into a `tempfile::NamedTempFile` and asserts on the
dataset, variables, and observation sections that are produced.