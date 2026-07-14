# checklog

A small Rust library that checks a log file line by line against user-supplied rules. Each line is reported with its 1-indexed line number, its content (with the trailing newline stripped), and whether it passed the check.

## Features

- **Two kinds of rules**: literal substring **keywords** and regular-expression **patterns**.
- **Issue and whitelist lists**: a line that matches any whitelist rule is unconditionally passed; a line that doesn't match any whitelist rule is then checked against the issue rules.
- **Whitelist always wins** — a line that matches both a whitelist rule and an issue rule still passes, regardless of the relative order of the two lists in your `Config`.
- **First hit wins** within a list — the order of your `Vec<String>` is the scan order.
- **Case sensitivity** is opt-in; the default is case-insensitive matching for both keywords and patterns. Per-pattern opt-out (e.g. `(?-i)`) is also honoured.
- **1-indexed line numbers** in the results.
- **No panics** on bad input. Invalid regex and I/O errors are returned as `Err`.

## Quick start

```rust
use checklog::{Checker, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config {
        issue_keywords: vec!["ERROR".into()],
        issue_patterns: vec![r"panic|abort".into()],
        whitelist_keywords: vec!["retrying".into()],
        whitelist_patterns: vec![],
        case_sensitive: false,
    };

    let checker = Checker::new(&cfg)?;
    let results = checker.check_file("app.log")?;

    for r in &results {
        let status = if r.passed { "OK  " } else { "FAIL" };
        println!("{:>4}  {}  {}", r.line_number, status, r.content);
    }
    Ok(())
}
```

`check_str` runs the same logic over an in-memory string (useful for tests and for content from `read_to_string`):

```rust
let results = checker.check_str("INFO ok\nERROR boom\nINFO done");
// results[1].passed == false
```

## API

### `Config`

```rust
pub struct Config {
    pub issue_keywords: Vec<String>,
    pub issue_patterns: Vec<String>,
    pub whitelist_keywords: Vec<String>,
    pub whitelist_patterns: Vec<String>,
    pub case_sensitive: bool,
}
```

`Config::new()` returns the all-defaults config (empty lists, `case_sensitive = false`). The struct also derives `Default`.

### `Checker`

```rust
impl Checker {
    pub fn new(config: &Config) -> Result<Self, CheckError>;
    pub fn check_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogResult>, CheckError>;
    pub fn check_str(&self, input: &str) -> Vec<LogResult>;
}
```

- `Checker::new` compiles the regex patterns once. Returns `Err(CheckError::Regex(_))` on the first invalid pattern.
- `check_file` reads the file line by line. Returns `Err(CheckError::Io(_))` on any I/O failure (including file-not-found).
- `check_str` runs the same logic over a `&str`.

### `LogResult`

```rust
pub struct LogResult {
    pub line_number: usize,   // 1-indexed
    pub content: String,      // trailing \n / \r\n stripped
    pub passed: bool,
}
```

### `CheckError`

```rust
pub enum CheckError {
    Io(std::io::Error),
    Regex(regex::Error),
}
```

`CheckError` implements `std::error::Error`; the wrapped error is the `source()`.

## Matching rules

For each line, in order:

1. **Whitelist keyword** as a substring (case-insensitive unless `case_sensitive = true`).
2. **Whitelist regex** match.
3. If any of those hit, the line is **passed** and we move on.
4. Otherwise, **issue keyword** as a substring.
5. **Issue regex** match.
6. If any of those hit, the line is **failed**.
7. Otherwise, the line is **passed** (clean).

A line that matches both a whitelist and an issue rule still passes — whitelist always wins.

## Line endings

Both `check_file` and `check_str` strip trailing `\n` and `\r\n`. Empty input returns an empty `Vec`. A trailing newline does not produce a trailing empty line in the output.
