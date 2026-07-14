# `checklog` Library — Design

**Date:** 2026-07-14
**Crate:** `crates/checklog`
**Status:** Approved (brainstorm complete)

## Purpose

A small Rust library that checks a log file line by line against user-supplied rules. Each line is reported with its 1-indexed line number, its content (with trailing newline stripped), and whether it passed the check.

Two kinds of rules are supported, separately for "issue" and "whitelist":

- **Keywords** — literal substring matches.
- **Patterns** — regular expressions.

A line that matches any whitelist rule is unconditionally **passed**. A line that does not match any whitelist rule is then checked against the issue rules; if any issue rule matches, the line **fails**. A line that matches no rule of either kind **passes** (it is treated as "clean").

## Module layout

```
crates/checklog/
├── Cargo.toml                # already exists, uses workspace regex
└── src/
    ├── lib.rs                # public re-exports
    ├── config.rs             # Config struct
    └── checker.rs            # Checker + LogResult + CheckError
```

`lib.rs` re-exports the public surface so callers use `checklog::{Config, Checker, CheckError, LogResult}`. The existing default `add` stub is removed. No `mod.rs` files (2024-edition file-hierarchy style).

## Public API

```rust
// config.rs
pub struct Config {
    pub issue_keywords: Vec<String>,        // literal substrings
    pub issue_patterns: Vec<String>,        // regex strings
    pub whitelist_keywords: Vec<String>,    // literal substrings
    pub whitelist_patterns: Vec<String>,    // regex strings
    pub case_sensitive: bool,               // default false
}

impl Config {
    pub fn new() -> Self;  // empty lists, case_sensitive = false
}

// checker.rs
pub struct LogResult {
    pub line_number: usize,   // 1-indexed
    pub content: String,      // line with trailing \n / \r\n stripped
    pub passed: bool,
}

pub struct Checker { /* private: compiled keyword sets + regex sets */ }

impl Checker {
    pub fn new(config: &Config) -> Result<Self, CheckError>;
    pub fn check_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogResult>, CheckError>;
    pub fn check_str(&self, input: &str) -> Vec<LogResult>;
}

pub enum CheckError {
    Io(io::Error),
    Regex(regex::Error),
}

// lib.rs
pub use config::Config;
pub use checker::{Checker, CheckError, LogResult};
```

## Data flow and matching rules

### Compile time (`Checker::new`)

1. For each entry in `issue_patterns`, compile a `Regex`. When `case_sensitive = false`, prefix the user's pattern with `(?i)` so the whole pattern is case-insensitive. When `case_sensitive = true`, the pattern is compiled as-is — the user can still opt in to case-insensitivity inside one specific pattern by writing `(?i)` themselves (the rightmost flag in scope wins, so a user-written `(?-i)` re-enables sensitivity locally). Return `Err(CheckError::Regex(_))` on the first invalid pattern.
2. Same for `whitelist_patterns`.
3. Keywords are stored as-is, lowercased if `!case_sensitive`, for cheap substring search.

### Per line (`check_file` and `check_str`)

1. Strip the trailing `\n` or `\r\n` (`BufRead::lines()` already does this for `check_file`; `check_str` splits on `\n` and trims a trailing `\r`).
2. **Whitelist wins.** Check in this order; first hit short-circuits and the line is marked `passed = true`:
   1. Any whitelist keyword matches as a substring (case-insensitive unless `case_sensitive`).
   2. Any whitelist regex matches.
3. Otherwise, **check issues.** First hit short-circuits and the line is marked `passed = false`:
   1. Any issue keyword matches as a substring.
   2. Any issue regex matches.
4. If no rule of either kind matches, the line is `passed = true` (clean).

### Line numbering

1-indexed. The first line emitted by `BufRead::lines()` is `line_number = 1`.

### Order within a list

For both keywords and patterns, the user's `Vec` order is the scan order; first hit wins. This makes the order observable and gives the user control.

### Empty line

`""` has no substring hits and no regex hits, so it is `passed = true`. This is a consequence of the rules; no special case is needed.

### Whitelist beats issue

If a line matches a whitelist rule and an issue rule, the whitelist always wins, regardless of the relative order of `whitelist_*` and `issue_*` in the user's `Config`. This is a documented guarantee.

## Error handling

- **Invalid regex in config:** returned from `Checker::new(&Config) -> Result<Self, CheckError>`. The caller decides how to react.
- **I/O errors reading the file:** returned from `check_file(...) -> Result<Vec<LogResult>, CheckError>`. The underlying `io::Error` is wrapped in `CheckError::Io`.
- **Partial reads:** if reading the file fails partway through, the IO error is propagated and no partial `Vec` is returned.
- **No panics** in the public API on user input.
- `CheckError` is a small manual enum with `std::fmt::Display` and `std::error::Error` impls. No external error-crate dependency.

## Testing strategy

Tests are written first (TDD) and live in `#[cfg(test)] mod tests` inside each module.

### `config.rs` tests

- `Config::new()` defaults: all four lists are empty; `case_sensitive` is `false`.

### `checker.rs` tests (all use `check_str` for hermetic, deterministic input)

1. No rules — every line passes, including the empty line.
2. Single issue keyword — only lines containing it fail.
3. Single issue pattern (regex) — matching lines fail.
4. Multiple keywords and multiple patterns — first match in the user's order wins within a list.
5. Whitelist skips a line that would otherwise fail.
6. Whitelist beats issue — a line matching both a whitelist rule and an issue rule still passes.
7. Case-insensitive default — `"ERROR"` and `"error"` both fail under a rule for `"error"`.
8. `case_sensitive = true` — `"ERROR"` does *not* fail under a rule for `"error"`.
9. Line numbering is 1-indexed and matches the line's position.
10. Trailing newline is stripped from `content` (no `\n` in the result).
11. Invalid regex — `Checker::new` returns `Err(CheckError::Regex(_))`.
12. Missing file — `check_file` returns `Err(CheckError::Io(_))`.
13. Realistic log fixture — multi-line input with a mix of passing, failing, and whitelisted-but-also-issued lines; assert the full expected `Vec<LogResult>`.

## Out of scope (for this iteration)

- A CLI binary. (The user confirmed: library only.)
- Builder-pattern API. (User chose the `Config` struct style.)
- A `reason` field on `LogResult`. (User chose pass/fail only.)
- Streaming/iterator return from `check_file`. (User chose `Vec<LogResult>`; `check_str` exists for the common in-memory case.)
- Per-line error reporting (e.g. an I/O error mid-file). Partial reads abort the whole call.
- `thiserror` or `anyhow`. The error enum is small enough to impl manually.
