//! Log-checker logic: compiles the user's rules and applies them to input.

use crate::Config;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Outcome for a single log line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogResult {
    /// 1-indexed line number.
    pub line_number: usize,
    /// The line text with the trailing `\n` / `\r\n` stripped.
    pub content: String,
    /// `true` if the line passed the check.
    pub passed: bool,
}

/// Errors returned by the checker.
#[derive(Debug, Error)]
pub enum CheckError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid regex: {0}")]
    Regex(#[from] regex::Error),
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
            Ok(regex::Regex::new(&source)?)
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

    /// Reserved for Task 3.
    pub fn check_str(&self, input: &str) -> Vec<LogResult> {
        // `str::lines()` matches `BufRead::lines()` semantics: empty input
        // yields no results, and a trailing newline does not produce a
        // trailing empty line. It also strips both `\n` and `\r\n`.
        input
            .lines()
            .enumerate()
            .map(|(i, line)| {
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

    /// Reserved for Task 4.
    pub fn check_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogResult>, CheckError> {
        use std::io::{BufRead, BufReader};

        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut out = Vec::new();
        for (i, line) in reader.lines().enumerate() {
            // BufRead::lines() strips the trailing '\n' / '\r\n' for us.
            let content = line?;
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
}

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
        assert_eq!(results, vec![passed(1, "a"), passed(2, "b")]);
    }
}
