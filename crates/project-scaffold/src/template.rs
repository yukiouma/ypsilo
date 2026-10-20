use crate::{Result, ScaffoldError};
use std::collections::HashMap;

/// What a template resolved to.
enum Substitution {
    /// Every placeholder had a value.
    Resolved(String),
    /// A placeholder named a key that `vars` does not hold.
    Unresolved(String),
}

/// Replace every `{{ key }}` placeholder in `template` with the matching entry in `vars`.
///
/// A key that `vars` does not hold leaves the whole template unresolved, and the
/// result is `None` — the field the template belongs to is null, rather than half
/// rendered. Use [`render_required`] where a missing value is an error instead.
pub fn render(template: &str, vars: &HashMap<String, String>) -> Option<String> {
    match substitute(template, vars) {
        Substitution::Resolved(value) => Some(value),
        Substitution::Unresolved(_) => None,
    }
}

/// Replace every `{{ key }}` placeholder in `template` with the matching entry in
/// `vars`, failing on a key that `vars` does not hold.
pub fn render_required(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    match substitute(template, vars) {
        Substitution::Resolved(value) => Ok(value),
        Substitution::Unresolved(key) => Err(ScaffoldError::UnknownTemplateKey {
            key,
            template: template.to_string(),
        }),
    }
}

/// Substitute every placeholder in `template`, stopping at the first key that
/// `vars` does not hold.
fn substitute(template: &str, vars: &HashMap<String, String>) -> Substitution {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        let (head, tail) = rest.split_at(start);
        out.push_str(head);

        // No closing braces left: the remainder carries no placeholder, keep it verbatim.
        let Some(end) = tail.find("}}") else {
            out.push_str(tail);
            return Substitution::Resolved(out);
        };

        // `{{` and `}}` are ASCII, so `start` and `end` are always char boundaries.
        let key = tail[2..end].trim();
        match vars.get(key) {
            Some(value) => out.push_str(value),
            None => return Substitution::Unresolved(key.to_string()),
        }

        rest = &tail[end + 2..];
    }

    out.push_str(rest);
    Substitution::Resolved(out)
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
        assert_eq!(render("documents", &vars()), Some("documents".to_string()));
    }

    #[test]
    fn substitutes_a_single_placeholder() {
        assert_eq!(render("{{ study }}", &vars()), Some("AK-001".to_string()));
    }

    #[test]
    fn substitutes_a_placeholder_embedded_in_surrounding_text() {
        assert_eq!(
            render("{{ study }} Data Issue Log.xlsx", &vars()),
            Some("AK-001 Data Issue Log.xlsx".to_string())
        );
    }

    #[test]
    fn substitutes_several_placeholders_in_one_template() {
        assert_eq!(
            render("{{ study }}-{{ purpose }}", &vars()),
            Some("AK-001-final".to_string())
        );
    }

    #[test]
    fn tolerates_missing_whitespace_inside_the_braces() {
        assert_eq!(render("{{study}}", &vars()), Some("AK-001".to_string()));
    }

    #[test]
    fn leaves_the_field_null_when_the_key_is_unknown() {
        assert_eq!(render("{{ nope }}", &vars()), None);
    }

    #[test]
    fn leaves_the_whole_field_null_when_one_of_its_keys_is_unknown() {
        assert_eq!(render("{{ study }} {{ nope }}.xlsx", &vars()), None);
    }

    #[test]
    fn treats_an_unterminated_placeholder_as_literal_text() {
        assert_eq!(render("{{ study ", &vars()), Some("{{ study ".to_string()));
    }

    #[test]
    fn renders_a_known_key_as_required() {
        assert_eq!(render_required("{{ study }}", &vars()).unwrap(), "AK-001");
    }

    #[test]
    fn rejects_an_unknown_key_when_the_value_is_required() {
        let err = render_required("{{ nope }}", &vars()).unwrap_err();
        assert!(matches!(err, ScaffoldError::UnknownTemplateKey { ref key, .. } if key == "nope"));
    }
}
