use crate::{Result, ScaffoldError};
use std::collections::HashMap;

/// Replace every `{{ key }}` placeholder in `template` with the matching entry in `vars`.
///
/// An unknown key is an error rather than an empty string, so a typo in a config
/// surfaces immediately instead of producing a blank directory name.
pub fn render(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        let (head, tail) = rest.split_at(start);
        out.push_str(head);

        // No closing braces left: the remainder carries no placeholder, keep it verbatim.
        let Some(end) = tail.find("}}") else {
            out.push_str(tail);
            return Ok(out);
        };

        // `{{` and `}}` are ASCII, so `start` and `end` are always char boundaries.
        let key = tail[2..end].trim();
        match vars.get(key) {
            Some(value) => out.push_str(value),
            None => {
                return Err(ScaffoldError::UnknownTemplateKey {
                    key: key.to_string(),
                    template: template.to_string(),
                });
            }
        }

        rest = &tail[end + 2..];
    }

    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScaffoldError;

    fn vars() -> HashMap<String, String> {
        HashMap::from([
            ("study".to_string(), "AK-001".to_string()),
            ("purpose".to_string(), "final".to_string()),
        ])
    }

    #[test]
    fn returns_input_when_no_placeholder_present() {
        assert_eq!(render("documents", &vars()).unwrap(), "documents");
    }

    #[test]
    fn substitutes_a_single_placeholder() {
        assert_eq!(render("{{ study }}", &vars()).unwrap(), "AK-001");
    }

    #[test]
    fn substitutes_a_placeholder_embedded_in_surrounding_text() {
        assert_eq!(
            render("{{ study }} Data Issue Log.xlsx", &vars()).unwrap(),
            "AK-001 Data Issue Log.xlsx"
        );
    }

    #[test]
    fn substitutes_several_placeholders_in_one_template() {
        assert_eq!(
            render("{{ study }}-{{ purpose }}", &vars()).unwrap(),
            "AK-001-final"
        );
    }

    #[test]
    fn tolerates_missing_whitespace_inside_the_braces() {
        assert_eq!(render("{{study}}", &vars()).unwrap(), "AK-001");
    }

    #[test]
    fn rejects_an_unknown_key() {
        let err = render("{{ nope }}", &vars()).unwrap_err();
        assert!(matches!(err, ScaffoldError::UnknownTemplateKey { ref key, .. } if key == "nope"));
    }

    #[test]
    fn treats_an_unterminated_placeholder_as_literal_text() {
        assert_eq!(render("{{ study ", &vars()).unwrap(), "{{ study ");
    }
}
