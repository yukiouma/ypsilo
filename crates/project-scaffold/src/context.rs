use crate::{Result, ScaffoldError};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// The per-study inputs, before normalisation.
///
/// Generic over both the path type and the string type, so callers can pass `&str`,
/// `String`, `&Path`, or `PathBuf` without converting first. Both parameters default to
/// the owning types, so existing callers are unaffected.
#[derive(Debug, Clone)]
pub struct RawContext<P = PathBuf, S = String> {
    pub product: S,
    pub trial: S,
    pub purpose: S,
    pub root_path: P,
    pub project_root: Option<P>,
    pub study: Option<S>,
    pub config_path: P,
}

/// Validated, normalised inputs plus the template variables they resolve to.
#[derive(Debug, Clone)]
pub struct Context {
    /// Uppercased.
    pub product: String,
    /// Uppercased.
    pub trial: String,
    /// Unchanged.
    pub purpose: String,
    /// Verified to exist.
    pub root_path: PathBuf,
    /// Either the verified explicit path, or derived under `root_path`.
    pub project_root: PathBuf,
    /// Either the explicit value, or `{PRODUCT}-{TRIAL}`.
    pub study: String,
    /// Verified to exist.
    pub config_path: PathBuf,
}

impl Context {
    /// Normalise and validate `raw`.
    ///
    /// `product` and `trial` are uppercased, `study` and `project_root` are derived when
    /// absent, and `root_path`, an explicit `project_root`, and `config_path` are checked
    /// for existence. A derived `project_root` is not checked — scaffolding creates it.
    ///
    /// Accepts any types implementing [`AsRef<Path>`] and [`AsRef<str>`]; the resulting
    /// [`Context`] owns concrete [`PathBuf`]s and [`String`]s.
    pub fn build<P: AsRef<Path>, S: AsRef<str>>(raw: RawContext<P, S>) -> Result<Self> {
        let product = raw.product.as_ref().to_uppercase();
        let trial = raw.trial.as_ref().to_uppercase();

        let root_path: PathBuf = raw.root_path.as_ref().to_path_buf();
        if !root_path.exists() {
            return Err(ScaffoldError::RootPathNotFound(root_path));
        }

        let project_root = match raw.project_root {
            Some(path) => {
                let path: PathBuf = path.as_ref().to_path_buf();
                if !path.exists() {
                    return Err(ScaffoldError::ProjectRootNotFound(path));
                }
                path
            }
            None => root_path
                .join(product.to_lowercase())
                .join(trial.to_lowercase()),
        };

        let study = raw
            .study
            .map(|s| s.as_ref().to_string())
            .unwrap_or_else(|| format!("{product}-{trial}"));

        let config_path: PathBuf = raw.config_path.as_ref().to_path_buf();
        if !config_path.exists() {
            return Err(ScaffoldError::ConfigPathNotFound(config_path));
        }

        Ok(Self {
            product,
            trial,
            purpose: raw.purpose.as_ref().to_string(),
            root_path,
            project_root,
            study,
            config_path,
        })
    }

    /// The variables a config template may reference.
    pub fn template_vars(&self) -> HashMap<String, String> {
        HashMap::from([
            ("product".to_string(), self.product.clone()),
            ("trial".to_string(), self.trial.clone()),
            ("purpose".to_string(), self.purpose.clone()),
            (
                "root_path".to_string(),
                self.root_path.display().to_string(),
            ),
            (
                "project_root".to_string(),
                self.project_root.display().to_string(),
            ),
            ("study".to_string(), self.study.clone()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScaffoldError;
    use std::fs;
    use tempfile::TempDir;

    /// A `RawContext` rooted at a fresh temp dir, with both derived fields left unset.
    fn raw(root: &TempDir) -> RawContext {
        RawContext {
            product: "ak".to_string(),
            trial: "xx".to_string(),
            purpose: "final".to_string(),
            root_path: root.path().to_path_buf(),
            project_root: None,
            study: None,
            config_path: root.path().join("config.json"),
        }
    }

    /// Write an empty config file into `root` so `Context::build` finds it.
    fn touch_config(root: &TempDir) {
        fs::write(root.path().join("config.json"), "{}").unwrap();
    }

    #[test]
    fn uppercases_product_and_trial() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let context = Context::build(raw(&dir)).unwrap();
        assert_eq!(context.product, "AK");
        assert_eq!(context.trial, "XX");
    }

    #[test]
    fn leaves_purpose_untouched() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let context = Context::build(raw(&dir)).unwrap();
        assert_eq!(context.purpose, "final");
    }

    #[test]
    fn derives_project_root_from_lowercased_product_and_trial() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let context = Context::build(raw(&dir)).unwrap();
        assert_eq!(context.project_root, dir.path().join("ak").join("xx"));
    }

    #[test]
    fn derives_study_from_uppercased_product_and_trial() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let context = Context::build(raw(&dir)).unwrap();
        assert_eq!(context.study, "AK-XX");
    }

    #[test]
    fn keeps_an_explicit_study() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let mut input = raw(&dir);
        input.study = Some("AK-999".to_string());
        assert_eq!(Context::build(input).unwrap().study, "AK-999");
    }

    #[test]
    fn keeps_an_explicit_project_root_when_it_exists() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let existing = dir.path().join("somewhere");
        fs::create_dir(&existing).unwrap();
        let mut input = raw(&dir);
        input.project_root = Some(existing.clone());
        assert_eq!(Context::build(input).unwrap().project_root, existing);
    }

    #[test]
    fn rejects_a_missing_root_path() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let mut input = raw(&dir);
        input.root_path = dir.path().join("nope");
        assert!(matches!(
            Context::build(input),
            Err(ScaffoldError::RootPathNotFound(_))
        ));
    }

    #[test]
    fn rejects_a_missing_explicit_project_root() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let mut input = raw(&dir);
        input.project_root = Some(dir.path().join("nope"));
        assert!(matches!(
            Context::build(input),
            Err(ScaffoldError::ProjectRootNotFound(_))
        ));
    }

    #[test]
    fn rejects_a_missing_config_path() {
        let dir = TempDir::new().unwrap();
        assert!(matches!(
            Context::build(raw(&dir)),
            Err(ScaffoldError::ConfigPathNotFound(_))
        ));
    }

    #[test]
    fn exposes_every_field_as_a_template_variable() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let context = Context::build(raw(&dir)).unwrap();
        let vars = context.template_vars();
        assert_eq!(vars["product"], "AK");
        assert_eq!(vars["trial"], "XX");
        assert_eq!(vars["purpose"], "final");
        assert_eq!(vars["study"], "AK-XX");
        assert_eq!(vars["root_path"], dir.path().display().to_string());
        assert_eq!(
            vars["project_root"],
            context.project_root.display().to_string()
        );
    }
}

#[cfg(test)]
mod generic_path_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Callers should be able to hand over borrowed strings or `String`s without
    /// converting to `PathBuf` first.
    #[test]
    fn accepts_borrowed_str_paths() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.json"), "{}").unwrap();

        let root = dir.path().to_str().unwrap().to_string();
        let config_path = dir.path().join("config.json");
        let config_path = config_path.to_str().unwrap().to_string();

        let context = Context::build(RawContext {
            product: "ak".to_string(),
            trial: "xx".to_string(),
            purpose: "final".to_string(),
            root_path: root.as_str(),
            project_root: None,
            study: None,
            config_path: config_path.as_str(),
        })
        .unwrap();

        assert_eq!(context.project_root, dir.path().join("ak").join("xx"));
        assert_eq!(context.study, "AK-XX");
    }

    #[test]
    fn accepts_owned_string_paths() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.json"), "{}").unwrap();

        let explicit = dir.path().join("somewhere");
        fs::create_dir(&explicit).unwrap();

        let context = Context::build(RawContext {
            product: "ak".to_string(),
            trial: "xx".to_string(),
            purpose: "final".to_string(),
            root_path: dir.path().to_str().unwrap().to_string(),
            project_root: Some(explicit.to_str().unwrap().to_string()),
            study: Some("AK-999".to_string()),
            config_path: dir.path().join("config.json").to_str().unwrap().to_string(),
        })
        .unwrap();

        assert_eq!(context.project_root, explicit);
        assert_eq!(context.study, "AK-999");
    }
}

#[cfg(test)]
mod generic_str_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// The string fields should accept `&str` directly, with no `.to_string()`.
    #[test]
    fn accepts_borrowed_str_fields() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.json"), "{}").unwrap();
        let root = dir.path().to_str().unwrap().to_string();
        let config_path = dir.path().join("config.json").to_str().unwrap().to_string();

        let context = Context::build(RawContext {
            product: "ak",
            trial: "xx",
            purpose: "final",
            root_path: root.as_str(),
            project_root: None,
            study: None,
            config_path: config_path.as_str(),
        })
        .unwrap();

        assert_eq!(context.product, "AK");
        assert_eq!(context.trial, "XX");
        assert_eq!(context.purpose, "final");
        assert_eq!(context.study, "AK-XX");
    }

    /// `&str` paths and `&str` text together, plus an explicit borrowed `study`.
    #[test]
    fn mixes_borrowed_strs_for_text_and_paths() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.json"), "{}").unwrap();
        let explicit = dir.path().join("elsewhere");
        fs::create_dir(&explicit).unwrap();
        let root = dir.path().to_str().unwrap().to_string();
        let config_path = dir.path().join("config.json").to_str().unwrap().to_string();
        let explicit = explicit.to_str().unwrap().to_string();

        let context = Context::build(RawContext {
            product: "ak",
            trial: "xx",
            purpose: "final",
            root_path: root.as_str(),
            project_root: Some(explicit.as_str()),
            study: Some("AK-999"),
            config_path: config_path.as_str(),
        })
        .unwrap();

        assert_eq!(context.study, "AK-999");
        assert_eq!(context.project_root, explicit);
    }
}
