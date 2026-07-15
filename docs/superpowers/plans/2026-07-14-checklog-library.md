# `checklog` Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the `checklog` library per [docs/superpowers/specs/2026-07-14-checklog-library-design.md](docs/superpowers/specs/2026-07-14-checklog-library-design.md): a small library that checks a log file line by line against user-supplied keywords and regex patterns, returning per-line results.

**Architecture:** Three modules. `config` holds the `Config` struct (four `Vec<String>` rule lists plus a `case_sensitive` flag). `checker` owns the `Checker`, `LogResult`, and `CheckError` types; it pre-compiles the user's regex patterns at construction time, then exposes `check_str` (in-memory) and `check_file` (path-based) methods. `lib` declares the modules and re-exports the public surface. Whitelist rules always win over issue rules.

**Tech Stack:** Rust 2024 edition, `regex = "1.13"` (workspace dep, already present), `tempfile` as a dev-dependency for the file-I/O tests.

## Global Constraints

- Workspace lives at `/root/coding/project/ypsilo`; crate is `crates/checklog`.
- All Rust code uses **edition = "2024"** and follows the file-hierarchy style (no `mod.rs`).
- Workspace `Cargo.toml` already has `[workspace.dependencies] regex = "1.13.0"`. The `checklog` member already references it as `regex = { workspace = true }`.
- New common dependencies (here: `tempfile` as a dev-dep) are added to the workspace root `Cargo.toml` under `[workspace.dev-dependencies]`, then referenced from the member via `tempfile = { workspace = true }`.
- For new dependencies, prefer `cargo add <crate> --dev -p checklog` over hand-editing `Cargo.toml`. Verify the workspace root was updated; if not, hand-edit.
- All test code lives inside `#[cfg(test)] mod tests` blocks in the same file as the unit it tests.
- Every task ends with a commit. Conventional-commit messages, lowercase, scoped to the crate where it makes sense.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/checklog/src/lib.rs` | Module declarations and public re-exports. |
| `crates/checklog/src/config.rs` | `Config` struct, `Default`/`new`, tests for defaults. |
| `crates/checklog/src/checker.rs` | `Checker`, `LogResult`, `CheckError`, all matching logic, all matching tests. |

`Cargo.toml` is touched only to add the `tempfile` dev-dependency in Task 4.

---

## Task 1: Project skeleton + `Config`

**Files:**
- Modify: `crates/checklog/src/lib.rs` (replace default `add` stub with module decls + re-exports)
- Create: `crates/checklog/src/config.rs`
- (No test file: tests live in-module.)

**Interfaces produced (consumed by later tasks):**
- `pub struct Config` with public fields `issue_keywords: Vec<String>`, `issue_patterns: Vec<String>`, `whitelist_keywords: Vec<String>`, `whitelist_patterns: Vec<String>`, `case_sensitive: bool`.
- `impl Config { pub fn new() -> Self }` — empty lists, `case_sensitive = false`.
- `impl Default for Config` — same as `new()` (derive `Default`).
- Re-exported at crate root as `checklog::Config`.

- [ ] **Step 1: Replace `lib.rs` with module decls and re-exports**

Replace the entire contents of `crates/checklog/src/lib.rs` with:

```rust
mod config;
mod checker;

pub use config::Config;
pub use checker::{CheckError, Checker, LogResult};
```

- [ ] **Step 2: Create `config.rs` with `Config` and write the failing defaults test**

Create `crates/checklog/src/config.rs`:

```rust
//! User configuration for the log checker.

/// Rules the checker uses to decide whether each log line passes.
///
/// A line passes if it matches any whitelist rule. Otherwise it fails if it
/// matches any issue rule. Otherwise it passes (it is "clean").
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Substrings (case-insensitive by default) that flag a line as a failure.
    pub issue_keywords: Vec<String>,
    /// Regular expressions (case-insensitive by default) that flag a line as a failure.
    pub issue_patterns: Vec<String>,
    /// Substrings that, if matched, cause the line to be passed unconditionally.
    pub whitelist_keywords: Vec<String>,
    /// Regular expressions that, if matched, cause the line to be passed unconditionally.
    pub whitelist_patterns: Vec<String>,
    /// When `true`, all keyword and pattern matches are case-sensitive.
    /// Default: `false`.
    pub case_sensitive: bool,
}

impl Config {
    /// Returns a configuration with no rules and case-insensitive matching.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_defaults() {
        let cfg = Config::new();
        assert!(cfg.issue_keywords.is_empty());
        assert!(cfg.issue_patterns.is_empty());
        assert!(cfg.whitelist_keywords.is_empty());
        assert!(cfg.whitelist_patterns.is_empty());
        assert!(!cfg.case_sensitive);
    }

    #[test]
    fn default_matches_new() {
        let a = Config::new();
        let b = Config::default();
        assert_eq!(a.issue_keywords, b.issue_keywords);
        assert_eq!(a.issue_patterns, b.issue_patterns);
        assert_eq!(a.whitelist_keywords, b.whitelist_keywords);
        assert_eq!(a.whitelist_patterns, b.whitelist_patterns);
        assert_eq!(a.case_sensitive, b.case_sensitive);
    }
}
```

- [ ] **Step 3: Create a stub `checker.rs` so `lib.rs` compiles**

Create `crates/checklog/src/checker.rs` with the bare type definitions referenced by `lib.rs`. (Full implementations come in Tasks 2–4.)

```rust
//! Log-checker logic: compiles the user's rules and applies them to input.

use std::path::Path;

use crate::Config;

/// Outcome for a single log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogResult {
    /// 1-indexed line number.
    pub line_number: usize,
    /// The line text with the trailing `\n` / `\r\n` stripped.
    pub content: String,
    /// `true` if the line passed the check.
    pub passed: bool,
}

/// Errors returned by the checker.
#[derive(Debug)]
pub enum CheckError {
    Io(std::io::Error),
    Regex(regex::Error),
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckError::Io(e) => write!(f, "I/O error: {e}"),
            CheckError::Regex(e) => write!(f, "invalid regex: {e}"),
        }
    }
}

impl std::error::Error for CheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CheckError::Io(e) => Some(e),
            CheckError::Regex(e) => Some(e),
        }
    }
}

/// Compiles a [`Config`] and applies it to log content.
#[derive(Debug)]
pub struct Checker {
    pub(crate) issue_keywords: Vec<String>,
    pub(crate) issue_patterns: Vec<regex::Regex>,
    pub(crate) whitelist_keywords: Vec<String>,
    pub(crate) whitelist_patterns: Vec<regex::Regex>,
    pub(crate) case_sensitive: bool,
}

impl Checker {
    /// Reserved for Task 2.
    pub fn new(_config: &Config) -> Result<Self, CheckError> {
        unimplemented!("filled in by Task 2")
    }

    /// Reserved for Task 3.
    pub fn check_str(&self, _input: &str) -> Vec<LogResult> {
        unimplemented!("filled in by Task 3")
    }

    /// Reserved for Task 4.
    pub fn check_file<P: AsRef<Path>>(&self, _path: P) -> Result<Vec<LogResult>, CheckError> {
        unimplemented!("filled in by Task 4")
    }
}
```

> The `pub(crate)` visibility on `Checker`'s fields is used by later tasks' tests in the same crate.

- [ ] **Step 4: Build and run tests**

Run from the workspace root:

```bash
cargo build -p checklog
cargo test -p checklog
```

Expected:
- `cargo build` succeeds with no warnings.
- `cargo test` shows the two `config::tests` tests passing; `checker::tests` (if any) is empty for now.

- [ ] **Step 5: Commit**

```bash
git add crates/checklog/src/lib.rs crates/checklog/src/config.rs crates/checklog/src/checker.rs
git commit -m "feat(checklog): scaffold lib with Config and stub Checker"
```

---

## Task 2: `Checker::new` — compile rules and surface regex errors

**Files:**
- Modify: `crates/checklog/src/checker.rs`

**Interfaces produced (consumed by later tasks):**
- `Checker::new(&Config) -> Result<Self, CheckError>` compiles all four rule lists; lowercases keywords when `case_sensitive = false`; prefixes each pattern with `(?i)` when `case_sensitive = false`. Returns `Err(CheckError::Regex(_))` on the first invalid pattern.

- [ ] **Step 1: Write the failing tests for `Checker::new`**

Append to the `tests` module in `crates/checklog/src/checker.rs` (create the module if it doesn't exist yet):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn empty_cfg() -> Config {
        Config::new()
    }

    #[test]
    fn new_empty_config_succeeds() {
        let cfg = empty_cfg();
        let checker = Checker::new(&cfg).expect("empty config must compile");
        assert!(checker.issue_keywords.is_empty());
        assert!(checker.issue_patterns.is_empty());
        assert!(checker.whitelist_keywords.is_empty());
        assert!(checker.whitelist_patterns.is_empty());
        assert!(!checker.case_sensitive);
    }

    #[test]
    fn new_lowercases_keywords_when_case_insensitive() {
        let cfg = Config {
            issue_keywords: vec!["ERROR".into(), "Fatal".into()],
            ..empty_cfg()
        };
        let checker = Checker::new(&cfg).unwrap();
        assert_eq!(checker.issue_keywords, vec!["error", "fatal"]);
    }

    #[test]
    fn new_preserves_keywords_when_case_sensitive() {
        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            case_sensitive: true,
            ..empty_cfg()
        };
        let checker = Checker::new(&cfg).unwrap();
        assert_eq!(checker.issue_keywords, vec!["ERROR"]);
    }

    #[test]
    fn new_invalid_issue_pattern_returns_regex_error() {
        let cfg = Config {
            issue_patterns: vec!["(".into()],
            ..empty_cfg()
        };
        let err = Checker::new(&cfg).expect_err("invalid regex must fail");
        assert!(matches!(err, CheckError::Regex(_)));
    }

    #[test]
    fn new_invalid_whitelist_pattern_returns_regex_error() {
        let cfg = Config {
            whitelist_patterns: vec!["[unclosed".into()],
            ..empty_cfg()
        };
        let err = Checker::new(&cfg).expect_err("invalid regex must fail");
        assert!(matches!(err, CheckError::Regex(_)));
    }

    #[test]
    fn new_prefixes_patterns_with_case_insensitive_flag() {
        // Pattern that matches only "abc" case-insensitively. With (?i) prefix
        // the compiled pattern must match "ABC". With case_sensitive = true,
    // it must not.
        let cfg_ci = Config {
            issue_patterns: vec!["abc".into()],
            ..empty_cfg()
        };
        let checker_ci = Checker::new(&cfg_ci).unwrap();
        assert!(checker_ci.issue_patterns[0].is_match("ABC"));

        let cfg_cs = Config {
            issue_patterns: vec!["abc".into()],
            case_sensitive: true,
            ..empty_cfg()
        };
        let checker_cs = Checker::new(&cfg_cs).unwrap();
        assert!(!checker_cs.issue_patterns[0].is_match("ABC"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p checklog checker::
```

Expected: the new tests fail (the `new` body is still `unimplemented!()`).

- [ ] **Step 3: Implement `Checker::new`**

Replace the body of `Checker::new` in `crates/checklog/src/checker.rs` with:

```rust
pub fn new(config: &Config) -> Result<Self, CheckError> {
    let case_sensitive = config.case_sensitive;

    let normalise_kw = |kw: &String| -> String {
        if case_sensitive {
            kw.clone()
        } else {
            kw.to_lowercase()
        }
    };

    let issue_keywords = config.issue_keywords.iter().map(normalise_kw).collect();
    let whitelist_keywords = config.whitelist_keywords.iter().map(normalise_kw).collect();

    let compile = |pat: &String| -> Result<regex::Regex, CheckError> {
        let source = if case_sensitive {
            pat.clone()
        } else {
            format!("(?i){pat}")
        };
        regex::Regex::new(&source).map_err(CheckError::Regex)
    };

    let mut issue_patterns = Vec::with_capacity(config.issue_patterns.len());
    for p in &config.issue_patterns {
        issue_patterns.push(compile(p)?);
    }
    let mut whitelist_patterns = Vec::with_capacity(config.whitelist_patterns.len());
    for p in &config.whitelist_patterns {
        whitelist_patterns.push(compile(p)?);
    }

    Ok(Self {
        issue_keywords,
        issue_patterns,
        whitelist_keywords,
        whitelist_patterns,
        case_sensitive,
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p checklog checker::
```

Expected: all six new `Checker::new` tests pass; previous `config::tests` still pass.

- [ ] **Step 5: Commit**

```bash
git add crates/checklog/src/checker.rs
git commit -m "feat(checklog): compile rules in Checker::new"
```

---

## Task 3: `Checker::check_str` — per-line matching

**Files:**
- Modify: `crates/checklog/src/checker.rs`

**Interfaces produced (consumed by Task 4):**
- `Checker::check_str(&self, input: &str) -> Vec<LogResult>` — splits on `\n`, strips trailing `\r` from each line, returns one `LogResult` per line, 1-indexed, with `passed` set per the matching rules.
- `Checker::check_line(&self, line: &str) -> bool` (private helper) — returns `true` if the line passes.

- [ ] **Step 1: Write the failing tests for `check_str`**

Append to the `tests` module in `crates/checklog/src/checker.rs`:

```rust
    fn checker(cfg: Config) -> Checker {
        Checker::new(&cfg).expect("test config must compile")
    }

    fn passed(line_number: usize, content: &str) -> LogResult {
        LogResult {
            line_number,
            content: content.to_string(),
            passed: true,
        }
    }

    fn failed(line_number: usize, content: &str) -> LogResult {
        LogResult {
            line_number,
            content: content.to_string(),
            passed: false,
        }
    }

    #[test]
    fn check_str_no_rules_all_lines_pass() {
        let c = checker(empty_cfg());
        let results = c.check_str("a\nb\nc");
        assert_eq!(
            results,
            vec![passed(1, "a"), passed(2, "b"), passed(3, "c")]
        );
    }

    #[test]
    fn check_str_empty_input_returns_empty() {
        let c = checker(empty_cfg());
        let results = c.check_str("");
        assert!(results.is_empty());
    }

    #[test]
    fn check_str_empty_line_passes() {
        let c = checker(empty_cfg());
        let results = c.check_str("a\n\nb");
        assert_eq!(results[1].content, "");
        assert!(results[1].passed);
    }

    #[test]
    fn check_str_issue_keyword_marks_matching_line_failed() {
        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        let results = c.check_str("INFO ok\nERROR boom\nINFO done");
        assert_eq!(
            results,
            vec![
                passed(1, "INFO ok"),
                failed(2, "ERROR boom"),
                passed(3, "INFO done"),
            ]
        );
    }

    #[test]
    fn check_str_issue_pattern_marks_matching_line_failed() {
        let cfg = Config {
            issue_patterns: vec![r"panic|abort".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        let results = c.check_str("INFO ok\nworker panic\nINFO done\naborted task");
        assert_eq!(
            results,
            vec![
                passed(1, "INFO ok"),
                failed(2, "worker panic"),
                passed(3, "INFO done"),
                failed(4, "aborted task"),
            ]
        );
    }

    #[test]
    fn check_str_first_keyword_match_wins() {
        let cfg = Config {
            issue_keywords: vec!["foo".into(), "bar".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        // Both "foo" and "bar" appear; first in the list still flags the line as failed.
        let results = c.check_str("foo and bar");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn check_str_first_pattern_match_wins() {
        let cfg = Config {
            issue_patterns: vec![r"foo".into(), r"bar".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        let results = c.check_str("foo and bar");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn check_str_whitelist_keyword_skips_issue() {
        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            whitelist_keywords: vec!["retrying".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        let results = c.check_str("ERROR retrying failed");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn check_str_whitelist_pattern_skips_issue() {
        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            whitelist_patterns: vec![r"retrying|expected".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        let results = c.check_str("ERROR expected failure");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn check_str_whitelist_beats_issue_even_when_both_match() {
        // The line matches both an issue keyword and a whitelist pattern.
        // Whitelist must win regardless of which list is iterated "first" in Config.
        let cfg_a = Config {
            issue_keywords: vec!["ERROR".into()],
            whitelist_patterns: vec![r"retried".into()],
            ..empty_cfg()
        };
        assert!(checker(cfg_a.clone()).check_str("ERROR retried eventually")[0].passed);

        // Same data, swap the relative "position" of the two lists in Config.
        // (Field order is not observable; we still test the same outcome.)
        let cfg_b = Config {
            whitelist_patterns: vec![r"retried".into()],
            issue_keywords: vec!["ERROR".into()],
            ..empty_cfg()
        };
        assert!(checker(cfg_b).check_str("ERROR retried eventually")[0].passed);
    }

    #[test]
    fn check_str_case_insensitive_by_default() {
        let cfg = Config {
            issue_keywords: vec!["error".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        assert!(!c.check_str("ERROR boom")[0].passed);
        assert!(!c.check_str("error boom")[0].passed);
        assert!(!c.check_str("Error boom")[0].passed);
    }

    #[test]
    fn check_str_case_sensitive_opt_in() {
        let cfg = Config {
            issue_keywords: vec!["error".into()],
            case_sensitive: true,
            ..empty_cfg()
        };
        let c = checker(cfg);
        assert!(!c.check_str("error boom")[0].passed);
        assert!(c.check_str("ERROR boom")[0].passed);
        assert!(c.check_str("Error boom")[0].passed);
    }

    #[test]
    fn check_str_strips_trailing_newline() {
        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);
        let results = c.check_str("INFO ok\nERROR bad\n");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].content, "INFO ok");
        assert_eq!(results[1].content, "ERROR bad");
        assert!(!results[0].content.contains('\n'));
        assert!(!results[1].content.contains('\n'));
    }

    #[test]
    fn check_str_strips_trailing_crlf() {
        let c = checker(empty_cfg());
        let results = c.check_str("a\r\nb\r\n");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].content, "a");
        assert_eq!(results[1].content, "b");
    }

    #[test]
    fn check_str_realistic_fixture() {
        // Mix of: clean line, issue-keyword hit, whitelist-keyword hit,
        // clean line, issue-keyword hit (also matches a pattern but the
        // keyword fires first), clean line, and a line that matches BOTH
        // a whitelist keyword and an issue keyword (whitelist must win).
        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            issue_patterns: vec![r"panic|abort".into()],
            whitelist_keywords: vec!["retrying".into()],
            whitelist_patterns: vec![],
            case_sensitive: false,
        };
        let input = "\
2024-01-01 INFO startup
2024-01-01 ERROR something went wrong
2024-01-01 INFO retrying connection
2024-01-01 WARN deprecated API
2024-01-01 ERROR panic in worker
2024-01-01 INFO shutdown
2024-01-01 ERROR retrying failed
";
        let results = checker(cfg).check_str(input);
        let expected = vec![
            passed(1, "2024-01-01 INFO startup"),
            failed(2, "2024-01-01 ERROR something went wrong"),
            passed(3, "2024-01-01 INFO retrying connection"),
            passed(4, "2024-01-01 WARN deprecated API"),
            failed(5, "2024-01-01 ERROR panic in worker"),
            passed(6, "2024-01-01 INFO shutdown"),
            passed(7, "2024-01-01 ERROR retrying failed"),
        ];
        assert_eq!(results, expected);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p checklog checker::tests::check_str
```

Expected: all new `check_str` tests fail (the body is still `unimplemented!()`).

- [ ] **Step 3: Implement `check_str` and the private `check_line` helper**

Replace the bodies of `check_str` and add a `check_line` helper in `crates/checklog/src/checker.rs`:

```rust
    pub fn check_str(&self, input: &str) -> Vec<LogResult> {
        input
            .split('\n')
            .enumerate()
            .map(|(i, raw)| {
                // Split on '\n' already removed the '\n'. Strip a trailing '\r'
                // so CRLF input behaves the same as LF input.
                let line = raw.strip_suffix('\r').unwrap_or(raw);
                let line_number = i + 1;
                let passed = self.check_line(line);
                LogResult {
                    line_number,
                    content: line.to_string(),
                    passed,
                }
            })
            .collect()
    }

    fn check_line(&self, line: &str) -> bool {
        // For case-insensitive keyword search we lowercase the haystack once.
        // For regex matches we pass the original line (the (?i) flag is on
        // the pattern itself, so the regex engine does its own case folding).
        let haystack = if self.case_sensitive {
            line.to_string()
        } else {
            line.to_lowercase()
        };

        // Whitelist first — first hit short-circuits as a pass.
        for kw in &self.whitelist_keywords {
            if haystack.contains(kw) {
                return true;
            }
        }
        for re in &self.whitelist_patterns {
            if re.is_match(line) {
                return true;
            }
        }

        // Then issues — first hit short-circuits as a fail.
        for kw in &self.issue_keywords {
            if haystack.contains(kw) {
                return false;
            }
        }
        for re in &self.issue_patterns {
            if re.is_match(line) {
                return false;
            }
        }

        // Clean.
        true
    }
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p checklog
```

Expected: all `config::tests` and `checker::tests` (every new test in this plan) pass.

- [ ] **Step 5: Commit**

```bash
git add crates/checklog/src/checker.rs
git commit -m "feat(checklog): implement per-line matching in check_str"
```

---

## Task 4: `Checker::check_file` — file I/O wrapper

**Files:**
- Modify: workspace root `Cargo.toml` (add `tempfile` to `[workspace.dev-dependencies]`)
- Modify: `crates/checklog/Cargo.toml` (add `tempfile` as a dev-dep via workspace inheritance)
- Modify: `crates/checklog/src/checker.rs` (add `check_file` implementation and tests)

**Interfaces produced:**
- `Checker::check_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogResult>, CheckError>` — opens the file, reads line by line, applies `check_line` to each, returns the same per-line `Vec<LogResult>` shape as `check_str`. Returns `Err(CheckError::Io(_))` on any I/O error (including file-not-found).

- [ ] **Step 1: Add the `tempfile` dev-dependency**

From the workspace root, run:

```bash
cargo add tempfile --dev -p checklog
```

Inspect the workspace root `Cargo.toml` to confirm a `[workspace.dev-dependencies]` block with `tempfile = "..."` was added (cargo does this automatically for workspace members). Inspect `crates/checklog/Cargo.toml` to confirm a `[dev-dependencies]` block with `tempfile = { workspace = true }` was added. If either is missing, hand-edit so the relationship is `tempfile = { workspace = true }` in the member and `tempfile = "<version>"` in the workspace root.

- [ ] **Step 2: Build to verify the dep is wired correctly**

```bash
cargo build -p checklog --tests
```

Expected: builds clean, no warnings.

- [ ] **Step 3: Write the failing tests for `check_file`**

Append to the `tests` module in `crates/checklog/src/checker.rs`:

```rust
    #[test]
    fn check_file_missing_returns_io_error() {
        let c = checker(empty_cfg());
        let result = c.check_file("/no/such/path/__checklog_does_not_exist.log");
        assert!(matches!(result, Err(CheckError::Io(_))));
    }

    #[test]
    fn check_file_reads_lines_with_one_indexed_numbers() {
        use std::io::Write;

        let cfg = Config {
            issue_keywords: vec!["ERROR".into()],
            ..empty_cfg()
        };
        let c = checker(cfg);

        let mut tmp = tempfile::NamedTempFile::new().expect("create tempfile");
        writeln!(tmp, "INFO startup").unwrap();
        writeln!(tmp, "ERROR failed").unwrap();
        writeln!(tmp, "INFO done").unwrap();

        let results = c.check_file(tmp.path()).expect("file must be readable");
        assert_eq!(
            results,
            vec![
                passed(1, "INFO startup"),
                failed(2, "ERROR failed"),
                passed(3, "INFO done"),
            ]
        );
    }

    #[test]
    fn check_file_strips_trailing_crlf() {
        use std::io::Write;

        let c = checker(empty_cfg());
        let mut tmp = tempfile::NamedTempFile::new().expect("create tempfile");
        // Write raw bytes including CRLF line endings.
        tmp.write_all(b"a\r\nb\r\n").unwrap();

        let results = c.check_file(tmp.path()).unwrap();
        assert_eq!(
            results,
            vec![passed(1, "a"), passed(2, "b")]
        );
    }
```

- [ ] **Step 4: Run tests to verify they fail**

```bash
cargo test -p checklog checker::tests::check_file
```

Expected: all three new `check_file` tests fail (the body is still `unimplemented!()`).

- [ ] **Step 5: Implement `check_file`**

Replace the body of `check_file` in `crates/checklog/src/checker.rs` with:

```rust
    pub fn check_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogResult>, CheckError> {
        use std::io::{BufRead, BufReader};

        let file = std::fs::File::open(path).map_err(CheckError::Io)?;
        let reader = BufReader::new(file);

        let mut out = Vec::new();
        for (i, line) in reader.lines().enumerate() {
            // BufRead::lines() strips the trailing '\n' / '\r\n' for us.
            let content = line.map_err(CheckError::Io)?;
            let line_number = i + 1;
            let passed = self.check_line(&content);
            out.push(LogResult {
                line_number,
                content,
                passed,
            });
        }
        Ok(out)
    }
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test -p checklog
```

Expected: every test in `config::tests` and `checker::tests` passes; `cargo build` is clean.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/checklog/Cargo.toml crates/checklog/src/checker.rs
git commit -m "feat(checklog): add check_file with tempfile-backed tests"
```

---

## Self-Review

**Spec coverage:**

| Spec section / requirement | Covered in |
|---|---|
| `Config` with the four `Vec<String>` fields and `case_sensitive: bool` | Task 1 |
| `Config::new()` defaults (empty lists, `case_sensitive = false`) | Task 1 |
| `LogResult { line_number, content, passed }` with 1-indexed numbering and stripped newline | Tasks 1, 3, 4 |
| `CheckError::Io`, `CheckError::Regex` with `Display` and `Error` impls | Task 1 |
| `Checker::new` compiles regex, returns `Regex` error on bad pattern, lowercases keywords for case-insensitive mode, prefixes patterns with `(?i)` for case-insensitive mode | Task 2 |
| Whitelist beats issue | Task 3 (`check_str_whitelist_beats_issue_even_when_both_match`) |
| Whitelist keyword first, then whitelist pattern; then issue keyword, then issue pattern | Task 3 (the order in `check_line`) |
| First-hit-wins within a list | Task 3 (`check_str_first_keyword_match_wins`, `check_str_first_pattern_match_wins`) |
| Case-insensitive default, case-sensitive opt-in | Task 3 (`check_str_case_insensitive_by_default`, `check_str_case_sensitive_opt_in`) |
| `check_str` strips trailing `\n` and `\r\n` | Task 3 (`check_str_strips_trailing_newline`, `check_str_strips_trailing_crlf`) |
| `check_file` returns `Err(CheckError::Io(_))` on missing file | Task 4 (`check_file_missing_returns_io_error`) |
| Realistic log fixture with mix of pass/fail/whitelist-wins | Task 3 (`check_str_realistic_fixture`) |

**Type consistency:** `LogResult`, `CheckError`, `Checker`, `Config` field names and signatures are introduced in Task 1 and referenced unchanged in Tasks 2–4. Method names (`new`, `check_str`, `check_file`, `check_line`) are consistent across tasks.

**Placeholder scan:** No "TBD", "TODO", "implement later", or vague phrasing. Every code block contains the actual code; every test contains the actual assertions; every commit command is the actual command.

**No spec requirement is missing a task.**
