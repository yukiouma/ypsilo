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
