use crate::{Result, context::Context, error::ScaffoldError, template};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

/// The scaffold description, deserialised from a JSON file.
///
/// Field names are camelCase on the wire (`projectRoot`, `reference`, `migration`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScaffoldConfig {
    /// Where the tree is rooted. May contain `{{ project_root }}`.
    pub project_root: String,
    pub directories: Vec<Node>,
}

/// One entry in the scaffold tree: a directory, a copied file, or a migration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    /// File or directory name. May contain `{{ ... }}` placeholders.
    pub name: String,
    pub directory: bool,
    /// Absolute path to a file copied to `name`.
    #[serde(default)]
    pub reference: Option<PathBuf>,
    /// Copy a filtered set of files out of an existing directory into `name`.
    #[serde(default)]
    pub migration: Option<Migration>,
    #[serde(default)]
    pub children: Vec<Node>,
}

/// Copy the immediate children of `directory` whose extension appears in `filter`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Migration {
    /// Absolute path to the source directory.
    pub directory: PathBuf,
    /// Include-list of extensions, each written with its leading dot, e.g. `[".sas"]`.
    pub filter: Vec<String>,
}

impl ProjectScaffoldConfig {
    /// Read the config at `context.config_path` and resolve every `{{ ... }}` placeholder
    /// against the context, so the result is ready to hand to
    /// [`crate::project_scaffolding`].
    pub fn build(context: &Context) -> Result<Self> {
        let raw = fs::read_to_string(&context.config_path)
            .map_err(|e| ScaffoldError::io(&context.config_path, e))?;

        let mut config: Self =
            serde_json::from_str(&raw).map_err(|e| ScaffoldError::ConfigParse {
                path: context.config_path.clone(),
                source: e,
            })?;

        config.resolve(&context.template_vars())?;
        Ok(config)
    }

    fn resolve(&mut self, vars: &HashMap<String, String>) -> Result<()> {
        self.project_root = template::render(&self.project_root, vars)?;
        for node in &mut self.directories {
            node.resolve(vars)?;
        }
        Ok(())
    }
}

impl Node {
    fn resolve(&mut self, vars: &HashMap<String, String>) -> Result<()> {
        self.name = template::render(&self.name, vars)?;
        for child in &mut self.children {
            child.resolve(vars)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScaffoldError, context::RawContext};
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn context_for(dir: &TempDir, config: &str) -> Context {
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, config).unwrap();
        Context::build(RawContext {
            product: "ak".to_string(),
            trial: "xx".to_string(),
            purpose: "final".to_string(),
            root_path: dir.path().to_path_buf(),
            project_root: None,
            study: None,
            config_path,
        })
        .unwrap()
    }

    const NESTED: &str = r#"{
        "projectRoot": "{{ project_root }}",
        "directories": [
            {
                "name": "documents",
                "directory": true,
                "children": [
                    {
                        "name": "specs",
                        "directory": true,
                        "children": [
                            {
                                "name": "{{ study }} SDTM Specification v1.0.xlsx",
                                "directory": false,
                                "reference": "/tmp/ref.xlsx"
                            }
                        ]
                    }
                ]
            }
        ]
    }"#;

    #[test]
    fn resolves_the_project_root_placeholder() {
        let dir = TempDir::new().unwrap();
        let context = context_for(&dir, NESTED);
        let config = ProjectScaffoldConfig::build(&context).unwrap();
        assert_eq!(
            config.project_root,
            context.project_root.display().to_string()
        );
    }

    #[test]
    fn resolves_placeholders_at_every_depth() {
        let dir = TempDir::new().unwrap();
        let context = context_for(&dir, NESTED);
        let config = ProjectScaffoldConfig::build(&context).unwrap();
        assert_eq!(config.directories[0].name, "documents");
        let specs = &config.directories[0].children[0];
        assert_eq!(specs.name, "specs");
        assert_eq!(specs.children[0].name, "AK-XX SDTM Specification v1.0.xlsx");
    }

    #[test]
    fn keeps_reference_and_migration_fields_as_written() {
        let dir = TempDir::new().unwrap();
        let context = context_for(&dir, NESTED);
        let config = ProjectScaffoldConfig::build(&context).unwrap();
        let leaf = &config.directories[0].children[0].children[0];
        assert_eq!(leaf.reference.as_deref(), Some(Path::new("/tmp/ref.xlsx")));
        assert!(leaf.migration.is_none());
    }

    #[test]
    fn treats_absent_optional_node_fields_as_none_and_empty() {
        let dir = TempDir::new().unwrap();
        let context = context_for(
            &dir,
            r#"{"projectRoot": "{{ project_root }}", "directories": [
                {"name": "plain", "directory": true}
            ]}"#,
        );
        let config = ProjectScaffoldConfig::build(&context).unwrap();
        let plain = &config.directories[0];
        assert!(plain.reference.is_none());
        assert!(plain.migration.is_none());
        assert!(plain.children.is_empty());
    }

    #[test]
    fn parses_a_migration_block() {
        let dir = TempDir::new().unwrap();
        let context = context_for(
            &dir,
            r#"{"projectRoot": "{{ project_root }}", "directories": [
                {"name": "macros", "directory": true, "migration": {
                    "directory": "/tmp/general-marcos", "filter": [".sas"]
                }}
            ]}"#,
        );
        let config = ProjectScaffoldConfig::build(&context).unwrap();
        let migration = config.directories[0].migration.as_ref().unwrap();
        assert_eq!(migration.directory, Path::new("/tmp/general-marcos"));
        assert_eq!(migration.filter, vec![".sas".to_string()]);
    }

    #[test]
    fn reports_malformed_json_with_the_config_path() {
        let dir = TempDir::new().unwrap();
        let context = context_for(&dir, "{ not json");
        assert!(matches!(
            ProjectScaffoldConfig::build(&context),
            Err(ScaffoldError::ConfigParse { .. })
        ));
    }

    #[test]
    fn reports_an_unknown_placeholder_key() {
        let dir = TempDir::new().unwrap();
        let context = context_for(
            &dir,
            r#"{"projectRoot": "{{ project_root }}", "directories": [
                {"name": "{{ unknown_key }}", "directory": true}
            ]}"#,
        );
        assert!(matches!(
            ProjectScaffoldConfig::build(&context),
            Err(ScaffoldError::UnknownTemplateKey { .. })
        ));
    }
}
