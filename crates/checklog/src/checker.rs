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
