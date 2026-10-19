use crate::{Result, ScaffoldError};
use std::{collections::HashMap, path::PathBuf};

/// The per-study inputs, before normalisation.
#[derive(Debug, Clone)]
pub struct RawContext {
    pub product: String,
    pub trial: String,
    pub purpose: String,
    pub root_path: PathBuf,
    pub project_root: Option<PathBuf>,
    pub study: Option<String>,
    pub config_path: PathBuf,
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
    pub fn build(raw: RawContext) -> Result<Self> {
        let product = raw.product.to_uppercase();
        let trial = raw.trial.to_uppercase();

        if !raw.root_path.exists() {
            return Err(ScaffoldError::RootPathNotFound(raw.root_path.clone()));
        }

        let project_root = match raw.project_root {
            Some(path) => {
                if !path.exists() {
                    return Err(ScaffoldError::ProjectRootNotFound(path));
                }
                path
            }
            None => raw
                .root_path
                .join(product.to_lowercase())
                .join(trial.to_lowercase()),
        };

        let study = raw.study.unwrap_or_else(|| format!("{product}-{trial}"));

        if !raw.config_path.exists() {
            return Err(ScaffoldError::ConfigPathNotFound(raw.config_path.clone()));
        }

        Ok(Self {
            product,
            trial,
            purpose: raw.purpose,
            root_path: raw.root_path,
            project_root,
            study,
            config_path: raw.config_path,
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
