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

    /// Reserved for Task 3.
    pub fn check_str(&self, _input: &str) -> Vec<LogResult> {
        unimplemented!("filled in by Task 3")
    }

    /// Reserved for Task 4.
    pub fn check_file<P: AsRef<Path>>(&self, _path: P) -> Result<Vec<LogResult>, CheckError> {
        unimplemented!("filled in by Task 4")
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
}
