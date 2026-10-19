# Project Scaffold Crate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `project-scaffold` library that materialises a clinical trial project's directory scaffold from a JSON configuration.

**Architecture:** Four small modules behind one public function. `Context` normalises and validates the per-study inputs and exposes them as template variables; `template` resolves `{{ key }}` placeholders; `config` deserialises the JSON tree and renders every node name; `scaffold` walks the tree and creates directories and files. `project_scaffolding(config)` is the entry point.

**Tech Stack:** Rust 2024, `serde` + `serde_json`, `thiserror`, `tempfile` (dev).

**Design spec:** [`docs/superpowers/specs/2026-10-19-project-scaffold-design.md`](../specs/2026-10-19-project-scaffold-design.md)

## Global Constraints

- Every crate uses `edition = "2024"` and lives under `crates/`.
- Modules use the 2024 file-hierarchy style: **never** create `mod.rs`. `mod foo;` resolves to `foo.rs`.
- Shared dependency versions are declared once in the workspace root `Cargo.toml` under `[workspace.dependencies]`; child crates reference them with `<name>.workspace = true`. Never duplicate a version string in a child manifest.
- Add dependencies with `cargo add <crate>`, never by hand-editing `Cargo.toml`.
- Test commands run from the workspace root `/root/coding/project/ypsilo`.
- A test asserting on filesystem contents must build its tree inside a `tempfile::TempDir`, never in the repo.

**Compile-first ordering.** Task 1 creates every source file as a compiling stub whose bodies are
`unimplemented!()`. Each later task appends its tests to the stub (they fail by panicking), then
replaces the stub body with the real implementation. This keeps `cargo test` runnable from Task 1
onward, so every TDD step is real.

## File Structure

| File | Responsibility |
| --- | --- |
| `crates/project-scaffold/Cargo.toml` | manifest; deps via workspace |
| `crates/project-scaffold/src/lib.rs` | `Result` alias, `project_scaffolding()`, re-exports |
| `crates/project-scaffold/src/error.rs` | `ScaffoldError` enum + `ScaffoldError::io()` helper |
| `crates/project-scaffold/src/template.rs` | `render()` — `{{ key }}` substitution |
| `crates/project-scaffold/src/context.rs` | `RawContext`, `Context`, normalisation, `template_vars()` |
| `crates/project-scaffold/src/config.rs` | `ProjectScaffoldConfig`, `Node`, `Migration`, `build()` |
| `crates/project-scaffold/src/scaffold.rs` | recursive tree walk: mkdir / copy / skip |
| `crates/project-scaffold/tests/scaffolding.rs` | integration tests over the real `config.json` |
| `crates/project-scaffold/README.md` | crate usage |
| `.mock_data/project-scaffold/config.json` | modified: absolute `reference` / `migration.directory` paths |

---

### Task 1: Manifest, error type, stubs, and `Result` alias

**Files:**
- Create: `crates/project-scaffold/src/error.rs`
- Create: `crates/project-scaffold/src/lib.rs` (replaces the `cargo new` placeholder)
- Create: `crates/project-scaffold/src/template.rs` (stub)
- Create: `crates/project-scaffold/src/context.rs` (stub)
- Create: `crates/project-scaffold/src/config.rs` (stub)
- Create: `crates/project-scaffold/src/scaffold.rs` (stub)
- Modify: `Cargo.toml` (workspace root) — add `serde_json`

**Interfaces:**
- Consumes: nothing.
- Produces: `crate::Result<T>`, `crate::ScaffoldError`, `ScaffoldError::io(impl Into<PathBuf>, io::Error)`, and compiling stubs for `template::render`, `Context`, `ProjectScaffoldConfig`, `scaffold::build_tree`. Every later task replaces a stub body.

- [ ] **Step 1: Add `serde_json` to the workspace dependencies**

Run from the workspace root:

```bash
cargo add serde_json
```

Expected: Cargo appends `serde_json = "1.x.y"` under `[workspace.dependencies]` in the root
`Cargo.toml`. Verify with `grep serde_json Cargo.toml`.

- [ ] **Step 2: Write the crate manifest**

Replace the contents of `crates/project-scaffold/Cargo.toml` with:

```toml
[package]
name = "project-scaffold"
version = "0.1.0"
edition = "2024"

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

- [ ] **Step 3: Write the error type**

Create `crates/project-scaffold/src/error.rs`:

```rust
use std::{io, path::PathBuf};
use thiserror::Error;

/// Every failure mode of the project scaffold crate.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse config at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("unknown template variable `{{{{{key}}}}}` in `{template}`")]
    UnknownTemplateKey { key: String, template: String },

    #[error("root path does not exist: {0}")]
    RootPathNotFound(PathBuf),

    #[error("project root does not exist: {0}")]
    ProjectRootNotFound(PathBuf),

    #[error("config file does not exist: {0}")]
    ConfigPathNotFound(PathBuf),

    #[error("reference file does not exist: {0}")]
    ReferenceNotFound(PathBuf),

    #[error("migration directory does not exist: {0}")]
    MigrationDirNotFound(PathBuf),
}

impl ScaffoldError {
    /// Wrap an [`io::Error`] together with the path it occurred on, so failures deep
    /// in the tree stay identifiable.
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
```

Note: the design spec lists a separate `ConfigRead` variant. Reading the config file goes through
`fs::read_to_string`, whose failures are `io::Error`, so `Io { path }` covers that case with the
path already attached and the error's `source()` intact. `ConfigRead` is deliberately folded away.

- [ ] **Step 4: Write `lib.rs`**

Replace the contents of `crates/project-scaffold/src/lib.rs` with:

```rust
//! Scaffold the directory tree of a clinical trial data processing project.

mod config;
mod context;
mod error;
mod scaffold;
mod template;

pub use config::{Migration, Node, ProjectScaffoldConfig};
pub use context::{Context, RawContext};
pub use error::ScaffoldError;

use std::path::PathBuf;

/// Result type for every fallible operation in this crate.
pub type Result<T> = std::result::Result<T, ScaffoldError>;

/// Materialise the project scaffold described by `config`.
///
/// Every path in `config` has already had its `{{ ... }}` placeholders resolved by
/// [`ProjectScaffoldConfig::build`]. Paths that already exist are left untouched, so
/// calling this twice over the same project root is a no-op.
pub fn project_scaffolding(config: ProjectScaffoldConfig) -> Result<()> {
    let root = PathBuf::from(&config.project_root);
    scaffold::build_tree(&config.directories, &root)
}
```

- [ ] **Step 5: Write the four module stubs**

These compile but panic when called. Each later task replaces the `unimplemented!()` body
and adds tests in the same file.

`crates/project-scaffold/src/template.rs`:

```rust
use crate::error::Result;
use std::collections::HashMap;

pub fn render(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    unimplemented!()
}
```

`crates/project-scaffold/src/context.rs`:

```rust
use crate::error::Result;
use std::{collections::HashMap, path::PathBuf};

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

#[derive(Debug, Clone)]
pub struct Context {
    pub product: String,
    pub trial: String,
    pub purpose: String,
    pub root_path: PathBuf,
    pub project_root: PathBuf,
    pub study: String,
    pub config_path: PathBuf,
}

impl Context {
    pub fn build(raw: RawContext) -> Result<Self> {
        unimplemented!()
    }

    pub fn template_vars(&self) -> HashMap<String, String> {
        unimplemented!()
    }
}
```

`crates/project-scaffold/src/config.rs`:

```rust
use crate::{context::Context, error::Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScaffoldConfig {
    pub project_root: String,
    pub directories: Vec<Node>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub name: String,
    pub directory: bool,
    #[serde(default)]
    pub reference: Option<PathBuf>,
    #[serde(default)]
    pub migration: Option<Migration>,
    #[serde(default)]
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Migration {
    pub directory: PathBuf,
    pub filter: Vec<String>,
}

impl ProjectScaffoldConfig {
    pub fn build(context: &Context) -> Result<Self> {
        unimplemented!()
    }
}
```

`crates/project-scaffold/src/scaffold.rs`:

```rust
use crate::{config::Node, error::Result};
use std::path::Path;

pub fn build_tree(nodes: &[Node], base: &Path) -> Result<()> {
    unimplemented!()
}
```

- [ ] **Step 6: Verify the crate compiles**

Run:

```bash
cargo build -p project-scaffold
```

Expected: `Finished` with no errors and no warnings.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/project-scaffold/
git commit -m "$(cat <<'EOF'
feat(project-scaffold): scaffold manifest, error type, and module stubs

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Template substitution

**Files:**
- Modify: `crates/project-scaffold/src/template.rs` (replace the stub body, add tests)

**Interfaces:**
- Consumes: `crate::Result`, `crate::ScaffoldError::UnknownTemplateKey` (Task 1).
- Produces: `crate::template::render(template: &str, vars: &HashMap<String, String>) -> Result<String>`. Task 4 calls it to resolve node names.

- [ ] **Step 1: Write the failing tests**

Append to `crates/project-scaffold/src/template.rs`:

```rust

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
        assert_eq!(render("{{ study }}-{{ purpose }}", &vars()).unwrap(), "AK-001-final");
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p project-scaffold template
```

Expected: 7 failed, each panicking with `not implemented`.

- [ ] **Step 3: Write the minimal implementation**

In `crates/project-scaffold/src/template.rs`, replace the stub (including its `use` lines) with:

```rust
use crate::error::{Result, ScaffoldError};
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
```

Keep the `#[cfg(test)] mod tests` block appended below it.

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p project-scaffold template
```

Expected: 7 passed, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add crates/project-scaffold/src/template.rs
git commit -m "$(cat <<'EOF'
feat(project-scaffold): add template placeholder substitution

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: `Context` normalisation

**Files:**
- Modify: `crates/project-scaffold/src/context.rs` (replace the stub, add tests)

**Interfaces:**
- Consumes: `crate::Result`, `crate::ScaffoldError` (Task 1).
- Produces: `Context::build(RawContext) -> Result<Context>`, `Context::template_vars() -> HashMap<String, String>`. Task 4 consumes both.

- [ ] **Step 1: Write the failing tests**

Append to `crates/project-scaffold/src/context.rs`:

```rust

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
        assert!(matches!(Context::build(input), Err(ScaffoldError::RootPathNotFound(_))));
    }

    #[test]
    fn rejects_a_missing_explicit_project_root() {
        let dir = TempDir::new().unwrap();
        touch_config(&dir);
        let mut input = raw(&dir);
        input.project_root = Some(dir.path().join("nope"));
        assert!(matches!(Context::build(input), Err(ScaffoldError::ProjectRootNotFound(_))));
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
        assert_eq!(vars["project_root"], context.project_root.display().to_string());
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p project-scaffold context
```

Expected: 10 failed, each panicking with `not implemented`.

- [ ] **Step 3: Write the implementation**

In `crates/project-scaffold/src/context.rs`, keep the two struct definitions unchanged and
replace the `impl Context` block (including its `use` lines) with:

```rust
use crate::error::{Result, ScaffoldError};
use std::{collections::HashMap, path::PathBuf};

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
            ("root_path".to_string(), self.root_path.display().to_string()),
            (
                "project_root".to_string(),
                self.project_root.display().to_string(),
            ),
            ("study".to_string(), self.study.clone()),
        ])
    }
}
```

The file is a single module, so there must be exactly one `use crate::error::...` and one
`use std::...` block at the top — delete the stub's copies when adding these.

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p project-scaffold context
```

Expected: 10 passed, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add crates/project-scaffold/src/context.rs
git commit -m "$(cat <<'EOF'
feat(project-scaffold): add context normalisation and validation

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Configuration model and `build()`

**Files:**
- Modify: `crates/project-scaffold/src/config.rs` (replace the stub body, add tests)

**Interfaces:**
- Consumes: `crate::Context` and `Context::template_vars()` (Task 3), `crate::template::render` (Task 2), `crate::ScaffoldError::{io, ConfigParse}` (Task 1).
- Produces: `ProjectScaffoldConfig { project_root: String, directories: Vec<Node> }`, `Node { name, directory, reference, migration, children }`, `Migration { directory, filter }`, and `ProjectScaffoldConfig::build(&Context) -> Result<ProjectScaffoldConfig>`. Tasks 5–8 consume these.

- [ ] **Step 1: Write the failing tests**

Append to `crates/project-scaffold/src/config.rs`:

```rust

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::RawContext, ScaffoldError};
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
        assert_eq!(config.project_root, context.project_root.display().to_string());
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p project-scaffold config
```

Expected: 7 failed, each panicking with `not implemented`.

- [ ] **Step 3: Write the implementation**

In `crates/project-scaffold/src/config.rs`, keep the three struct definitions unchanged and
replace the `impl ProjectScaffoldConfig` block (including its `use` lines) with:

```rust
use crate::{
    context::Context,
    error::{Result, ScaffoldError},
    template,
};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

impl ProjectScaffoldConfig {
    /// Read the config at `context.config_path` and resolve every `{{ ... }}` placeholder
    /// against the context, so the result is ready to hand to
    /// [`crate::project_scaffolding`].
    pub fn build(context: &Context) -> Result<Self> {
        let raw = fs::read_to_string(&context.config_path)
            .map_err(|e| ScaffoldError::io(&context.config_path, e))?;

        let mut config: Self = serde_json::from_str(&raw).map_err(|e| ScaffoldError::ConfigParse {
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
```

Delete the stub's `use` lines when adding these — a module may declare `use` only once.

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p project-scaffold config
```

Expected: 7 passed, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add crates/project-scaffold/src/config.rs
git commit -m "$(cat <<'EOF'
feat(project-scaffold): add config model and template resolution

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: Directory and file creation

**Files:**
- Modify: `crates/project-scaffold/src/scaffold.rs` (replace the stub, add tests)

**Interfaces:**
- Consumes: `crate::config::Node` (Task 4), `crate::ScaffoldError::io` (Task 1).
- Produces: `crate::scaffold::build_tree(nodes: &[Node], base: &Path) -> Result<()>`, called by `project_scaffolding`. Tasks 6 and 7 add branches to the same walk.

- [ ] **Step 1: Write the failing tests**

Append to `crates/project-scaffold/src/scaffold.rs`:

```rust

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScaffoldError;
    use std::fs;
    use tempfile::TempDir;

    fn dir(name: &str) -> Node {
        Node {
            name: name.to_string(),
            directory: true,
            reference: None,
            migration: None,
            children: Vec::new(),
        }
    }

    fn file(name: &str) -> Node {
        Node {
            directory: false,
            ..dir(name)
        }
    }

    fn with_children(name: &str, children: Vec<Node>) -> Node {
        Node { children, ..dir(name) }
    }

    #[test]
    fn creates_a_single_directory() {
        let tmp = TempDir::new().unwrap();
        build_tree(&[dir("documents")], tmp.path()).unwrap();
        assert!(tmp.path().join("documents").is_dir());
    }

    #[test]
    fn creates_a_nested_tree() {
        let tmp = TempDir::new().unwrap();
        build_tree(
            &[with_children(
                "documents",
                vec![with_children(
                    "specs",
                    vec![file("AK-XX SDTM Specification v1.0.xlsx")],
                )],
            )],
            tmp.path(),
        )
        .unwrap();
        let leaf = tmp
            .path()
            .join("documents")
            .join("specs")
            .join("AK-XX SDTM Specification v1.0.xlsx");
        assert!(leaf.is_file());
    }

    #[test]
    fn creates_the_whole_project_root_when_it_is_missing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("ak").join("xx");
        build_tree(&[dir("documents")], &root).unwrap();
        assert!(root.join("documents").is_dir());
    }

    #[test]
    fn skips_a_directory_that_already_exists() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("documents").join("keep.txt"), b"keep").unwrap();

        build_tree(&[dir("documents")], tmp.path()).unwrap();

        assert_eq!(
            fs::read(tmp.path().join("documents").join("keep.txt")).unwrap(),
            b"keep"
        );
    }

    #[test]
    fn skips_a_file_that_already_exists_and_leaves_its_contents_alone() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("notes.txt"), b"original").unwrap();

        build_tree(&[file("notes.txt")], tmp.path()).unwrap();

        assert_eq!(fs::read(tmp.path().join("notes.txt")).unwrap(), b"original");
    }

    #[test]
    fn running_twice_is_a_no_op() {
        let tmp = TempDir::new().unwrap();
        let tree = vec![with_children("documents", vec![dir("crf")])];
        build_tree(&tree, tmp.path()).unwrap();
        build_tree(&tree, tmp.path()).unwrap();
        assert!(tmp.path().join("documents").join("crf").is_dir());
    }

    #[test]
    fn reports_the_path_of_a_node_it_could_not_create() {
        // A file where a directory must be: `create_dir_all` fails and names the path.
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("documents"), b"not a directory").unwrap();
        let err = build_tree(&[dir("documents")], tmp.path()).unwrap_err();
        assert!(matches!(err, ScaffoldError::Io { .. }));
        assert!(err.to_string().contains("documents"));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p project-scaffold scaffold
```

Expected: 7 failed, each panicking with `not implemented`.

- [ ] **Step 3: Write the implementation**

In `crates/project-scaffold/src/scaffold.rs`, replace the stub (including its `use` lines) with:

```rust
use crate::{
    config::Node,
    error::{Result, ScaffoldError},
};
use std::{
    fs,
    path::Path,
};

/// Materialise `nodes` beneath `base`.
///
/// Anything that already exists is left untouched, so this is safe to re-run.
pub fn build_tree(nodes: &[Node], base: &Path) -> Result<()> {
    for node in nodes {
        create_node(node, base)?;
    }
    Ok(())
}

fn create_node(node: &Node, base: &Path) -> Result<()> {
    let target = base.join(&node.name);

    if node.directory {
        create_dir(&target)?;
        return build_tree(&node.children, &target);
    }

    create_file(&target)
}

fn create_dir(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path).map_err(|e| ScaffoldError::io(path, e))
}

fn create_file(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::File::create(path)
        .map(|_| ())
        .map_err(|e| ScaffoldError::io(path, e))
}
```

Keep the `#[cfg(test)] mod tests` block appended below it.

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p project-scaffold
```

Expected: 31 passed, 0 failed (7 template + 10 context + 7 config + 7 scaffold).

- [ ] **Step 5: Commit**

```bash
git add crates/project-scaffold/src/scaffold.rs
git commit -m "$(cat <<'EOF'
feat(project-scaffold): create scaffold directories and files

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: `reference` file copying

**Files:**
- Modify: `crates/project-scaffold/src/scaffold.rs` — `create_node` dispatch and a new `copy_reference`

**Interfaces:**
- Consumes: `crate::config::Node::reference` (Task 4), `ScaffoldError::ReferenceNotFound` (Task 1).
- Produces: `scaffold::copy_reference(reference: &Path, target: &Path) -> Result<()>`.

- [ ] **Step 1: Write the failing tests**

Append these inside the existing `mod tests` block in `crates/project-scaffold/src/scaffold.rs`,
and add `use std::path::Path;` alongside the existing `use std::fs;`:

```rust
    fn reference_node(name: &str, reference: &Path) -> Node {
        Node {
            name: name.to_string(),
            directory: false,
            reference: Some(reference.to_path_buf()),
            migration: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn copies_a_reference_file_to_its_configured_name() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("template v1.0.xlsx"), b"workbook").unwrap();

        build_tree(
            &[reference_node(
                "AK-XX SDTM Specification v1.0.xlsx",
                &src.path().join("template v1.0.xlsx"),
            )],
            dst.path(),
        )
        .unwrap();

        let copied = dst.path().join("AK-XX SDTM Specification v1.0.xlsx");
        assert_eq!(fs::read(&copied).unwrap(), b"workbook");
    }

    #[test]
    fn creates_the_parent_directory_for_a_reference_file() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("template.xlsx"), b"x").unwrap();

        build_tree(
            &[reference_node("template.xlsx", &src.path().join("template.xlsx"))],
            &dst.path().join("specs"),
        )
        .unwrap();

        assert!(dst.path().join("specs").join("template.xlsx").is_file());
    }

    #[test]
    fn errors_when_the_reference_source_is_missing() {
        let dst = TempDir::new().unwrap();
        let err = build_tree(
            &[reference_node("out.xlsx", Path::new("/nonexistent/template.xlsx"))],
            dst.path(),
        )
        .unwrap_err();
        assert!(matches!(err, ScaffoldError::ReferenceNotFound(_)));
    }

    #[test]
    fn does_not_overwrite_an_existing_copied_file() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        fs::write(src.path().join("template.xlsx"), b"fresh").unwrap();
        fs::write(dst.path().join("template.xlsx"), b"edited").unwrap();

        build_tree(
            &[reference_node("template.xlsx", &src.path().join("template.xlsx"))],
            dst.path(),
        )
        .unwrap();

        assert_eq!(fs::read(dst.path().join("template.xlsx")).unwrap(), b"edited");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p project-scaffold scaffold
```

Expected: 4 failed.
`copies_a_reference_file_to_its_configured_name` and `does_not_overwrite_an_existing_copied_file`
fail because `create_file` writes an empty file instead of copying;
`creates_the_parent_directory_for_a_reference_file` fails because `fs::File::create` does not
create missing parents; `errors_when_the_reference_source_is_missing` fails because no
`ReferenceNotFound` is produced. The 7 tests from Task 5 still pass.

- [ ] **Step 3: Write the implementation**

In `create_node`, insert the `reference` branch immediately after `let target = ...`:

```rust
    if let Some(reference) = &node.reference {
        return copy_reference(reference, &target);
    }
```

Then append to `scaffold.rs`:

```rust
/// Copy `reference` to `target`, leaving an existing target alone.
fn copy_reference(reference: &Path, target: &Path) -> Result<()> {
    if !reference.exists() {
        return Err(ScaffoldError::ReferenceNotFound(reference.to_path_buf()));
    }
    if target.exists() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        create_dir(parent)?;
    }
    fs::copy(reference, target)
        .map(|_| ())
        .map_err(|e| ScaffoldError::io(target, e))
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p project-scaffold
```

Expected: 35 passed, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add crates/project-scaffold/src/scaffold.rs
git commit -m "$(cat <<'EOF'
feat(project-scaffold): copy reference files into the scaffold

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 7: `migration` copying

**Files:**
- Modify: `crates/project-scaffold/src/scaffold.rs` — `create_node` dispatch and a new `copy_migration`

**Interfaces:**
- Consumes: `crate::config::{Node::migration, Migration}` (Task 4), `ScaffoldError::MigrationDirNotFound` (Task 1).
- Produces: `scaffold::copy_migration(migration: &Migration, target: &Path) -> Result<()>`.

- [ ] **Step 1: Write the failing tests**

Append these inside the `mod tests` block in `crates/project-scaffold/src/scaffold.rs`, and
change the test module's `use crate::config::Node;` to `use crate::config::{Migration, Node};`:

```rust
    fn migration_node(name: &str, directory: &Path, filter: &[&str]) -> Node {
        Node {
            name: name.to_string(),
            directory: true,
            reference: None,
            migration: Some(Migration {
                directory: directory.to_path_buf(),
                filter: filter.iter().map(|f| f.to_string()).collect(),
            }),
            children: Vec::new(),
        }
    }

    /// A source directory holding two `.sas` files, one `.xlsx`, and a subdirectory.
    fn general_marcos_fixtures() -> TempDir {
        let src = TempDir::new().unwrap();
        fs::write(src.path().join("m-forest-v1-0.sas"), b"macro").unwrap();
        fs::write(src.path().join("m-seriesplot-v1-0 .sas"), b"macro").unwrap();
        fs::write(src.path().join("Macro category information.xlsx"), b"book").unwrap();
        fs::create_dir(src.path().join("nested")).unwrap();
        fs::write(src.path().join("nested").join("deep-v1.sas"), b"deep").unwrap();
        src
    }

    #[test]
    fn copies_only_files_matching_the_extension_filter() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();

        build_tree(&[migration_node("macros", src.path(), &[".sas"])], dst.path()).unwrap();

        let macros = dst.path().join("macros");
        assert!(macros.is_dir());
        assert!(macros.join("m-forest-v1-0.sas").is_file());
        assert!(macros.join("m-seriesplot-v1-0 .sas").is_file());
        assert!(!macros.join("Macro category information.xlsx").exists());
    }

    #[test]
    fn does_not_recurse_into_subdirectories_of_the_source() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();

        build_tree(&[migration_node("macros", src.path(), &[".sas"])], dst.path()).unwrap();

        assert!(!dst.path().join("macros").join("nested").exists());
    }

    #[test]
    fn creates_an_empty_directory_when_nothing_matches_the_filter() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();

        build_tree(&[migration_node("macros", src.path(), &[".txt"])], dst.path()).unwrap();

        let macros = dst.path().join("macros");
        assert!(macros.is_dir());
        assert_eq!(fs::read_dir(&macros).unwrap().count(), 0);
    }

    #[test]
    fn leaves_already_migrated_files_alone() {
        let src = general_marcos_fixtures();
        let dst = TempDir::new().unwrap();
        fs::create_dir(dst.path().join("macros")).unwrap();
        fs::write(
            dst.path().join("macros").join("m-forest-v1-0.sas"),
            b"local edit",
        )
        .unwrap();

        build_tree(&[migration_node("macros", src.path(), &[".sas"])], dst.path()).unwrap();

        assert_eq!(
            fs::read(dst.path().join("macros").join("m-forest-v1-0.sas")).unwrap(),
            b"local edit"
        );
    }

    #[test]
    fn errors_when_the_migration_source_is_missing() {
        let dst = TempDir::new().unwrap();
        let err = build_tree(
            &[migration_node("macros", Path::new("/nonexistent/macros"), &[".sas"])],
            dst.path(),
        )
        .unwrap_err();
        assert!(matches!(err, ScaffoldError::MigrationDirNotFound(_)));
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p project-scaffold scaffold
```

Expected: 2 failed —
`copies_only_files_matching_the_extension_filter` (no `.sas` files are copied) and
`errors_when_the_migration_source_is_missing` (no `MigrationDirNotFound` is produced).

The other three pass vacuously, because `create_node` ignores `migration` and falls through to the
`directory: true` branch, which creates the target directory and copies nothing: that already
satisfies "does not recurse", "empty when nothing matches", and "leaves existing files alone".
They earn their keep once the migration branch exists.

- [ ] **Step 3: Write the implementation**

In `create_node`, insert the `migration` branch immediately after `let target = ...`, above the
`reference` branch:

```rust
    if let Some(migration) = &node.migration {
        return copy_migration(migration, &target);
    }
```

Then append to `scaffold.rs`:

```rust
/// Copy the immediate children of `migration.directory` whose extension is in
/// `migration.filter` into `target`, which is created if absent.
///
/// The source directory is not searched recursively.
fn copy_migration(migration: &Migration, target: &Path) -> Result<()> {
    let source: &Path = &migration.directory;
    if !source.exists() {
        return Err(ScaffoldError::MigrationDirNotFound(source.to_path_buf()));
    }

    create_dir(target)?;

    let entries = fs::read_dir(source).map_err(|e| ScaffoldError::io(source, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| ScaffoldError::io(source, e))?;
        let path = entry.path();

        if !path.is_file() || !extension_is_allowed(&path, &migration.filter) {
            continue;
        }

        let destination = target.join(entry.file_name());
        if destination.exists() {
            continue;
        }
        fs::copy(&path, &destination)
            .map(|_| ())
            .map_err(|e| ScaffoldError::io(&destination, e))?;
    }

    Ok(())
}

/// Whether `path`'s extension appears in `filter`. Filter entries carry a leading dot.
fn extension_is_allowed(path: &Path, filter: &[String]) -> bool {
    let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let extension = format!(".{extension}");
    filter.iter().any(|allowed| allowed == &extension)
}
```

And update the module's imports to pull in `Migration`:

```rust
use crate::{
    config::{Migration, Node},
    error::{Result, ScaffoldError},
};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p project-scaffold
```

Expected: 40 passed, 0 failed.

- [ ] **Step 5: Run fmt and clippy**

Run:

```bash
cargo fmt -p project-scaffold && cargo clippy -p project-scaffold --all-targets -- -D warnings
```

Expected: clippy prints nothing and exits 0. Run `cargo test -p project-scaffold` again if `fmt`
reformatted anything.

- [ ] **Step 6: Commit**

```bash
git add crates/project-scaffold/src/scaffold.rs
git commit -m "$(cat <<'EOF'
feat(project-scaffold): migrate macro files via extension filter

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

### Task 8: Absolute mock paths and end-to-end integration test

**Files:**
- Modify: `.mock_data/project-scaffold/config.json`
- Create: `crates/project-scaffold/tests/scaffolding.rs`
- Create: `crates/project-scaffold/README.md`

**Interfaces:**
- Consumes: the whole crate public API — `RawContext`, `Context::build`, `ProjectScaffoldConfig::build`, `project_scaffolding`, `ScaffoldError` (Tasks 1–7).

- [ ] **Step 1: Rewrite the mock config's paths as absolute**

Replace each `reference` value and the `migration.directory` value in `.mock_data/project-scaffold/config.json` with an absolute path under the repository. The five values become:

| Field | New value |
| --- | --- |
| `documents → specs → SDTM specification` → `reference` | `/root/coding/project/ypsilo/.mock_data/project-scaffold/references/AKxxx-xxx SDTM Specification_template v1.0.xlsx` |
| `documents → specs → ADaM specification` → `reference` | `/root/coding/project/ypsilo/.mock_data/project-scaffold/references/AKxxx-xxx ADaM Specification_template v1.0.xlsx` |
| `documents → project management → data issues tracking log → Data Issue Log` → `reference` | `/root/coding/project/ypsilo/.mock_data/project-scaffold/references/CL-CSP-DOC-005-A02 数据问题记录表 Data Issue Log-V1.0.xlsx` |
| `stats → {{ purpose }} → utility → Validation Review Tracker` → `reference` | `/root/coding/project/ypsilo/.mock_data/project-scaffold/references/CL-CSP-DOC-002-A01 质量控制追踪表 Validation Review Tracker-V2.0.xlsx` |
| `stats → {{ purpose }} → macros` → `migration.directory` | `/root/coding/project/ypsilo/.mock_data/project-scaffold/references/general-marcos` |

Leave every other field — including `"filter": [".sas"]` — untouched.

Verify none are still relative:

```bash
grep -n '"reference"\|"directory": ".mock_data' .mock_data/project-scaffold/config.json
```

Expected: four `reference` lines, all starting with `/root/coding/project/ypsilo/.mock_data/`.

- [ ] **Step 2: Write the integration test**

Create `crates/project-scaffold/tests/scaffolding.rs`:

```rust
//! End-to-end scaffolding against the repository's own `config.json`.

use project_scaffold::{
    Context, ProjectScaffoldConfig, RawContext, ScaffoldError, project_scaffolding,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const CONFIG: &str = "/root/coding/project/ypsilo/.mock_data/project-scaffold/config.json";
const REFERENCES: &str =
    "/root/coding/project/ypsilo/.mock_data/project-scaffold/references";
const GENERAL_MARCOS: &str =
    "/root/coding/project/ypsilo/.mock_data/project-scaffold/references/general-marcos";

fn raw_context(root: &Path, project_root: Option<PathBuf>, config_path: PathBuf) -> RawContext {
    RawContext {
        product: "ak".to_string(),
        trial: "xx".to_string(),
        purpose: "final".to_string(),
        root_path: root.to_path_buf(),
        project_root,
        study: None,
        config_path,
    }
}

/// Scaffold the real config into `tmp`, rooted at `{tmp}/ak/xx`, study `AK-XX`.
fn scaffold(tmp: &TempDir) -> PathBuf {
    let context =
        Context::build(raw_context(tmp.path(), None, PathBuf::from(CONFIG))).unwrap();
    let root = context.project_root.clone();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();
    root
}

fn file_names(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn creates_the_expected_top_level_tree() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    assert!(root.join("documents").join("protocol").is_dir());
    assert!(root.join("documents").join("crf").is_dir());
    assert!(root.join("documents").join("sap").is_dir());
    assert!(root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx")
        .is_file());
    assert!(root
        .join("documents")
        .join("specs")
        .join("AK-XX ADaM Specification v1.0.xlsx")
        .is_file());
    assert!(root
        .join("documents")
        .join("project management")
        .join("communication log")
        .is_dir());
}

#[test]
fn interpolates_the_purpose_into_the_stats_tree() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    assert!(root
        .join("stats")
        .join("final")
        .join("product")
        .join("program")
        .join("adam")
        .is_dir());
    assert!(root.join("stats").join("cutoff").join("rawdata").is_dir());
}

#[test]
fn copies_reference_files_under_their_interpolated_names() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    let sdtm = root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx");
    let template = Path::new(REFERENCES).join("AKxxx-xxx SDTM Specification_template v1.0.xlsx");
    assert_eq!(
        fs::metadata(&sdtm).unwrap().len(),
        fs::metadata(&template).unwrap().len()
    );

    let log = root
        .join("documents")
        .join("project management")
        .join("data issues tracking log")
        .join("AK-XX Data Issue Log.xlsx");
    assert!(log.is_file());
}

#[test]
fn migrates_only_the_sas_macros() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    let macros = root.join("stats").join("final").join("macros");
    let copied = file_names(&macros);

    let expected: BTreeSet<String> = fs::read_dir(GENERAL_MARCOS)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("sas")
        })
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    assert!(!expected.is_empty(), "fixture should contain .sas files");
    assert_eq!(copied, expected);
    assert!(!macros.join("Macro category information.xlsx").exists());
    assert!(!macros.join("update history.xlsx").exists());
}

#[test]
fn a_second_run_changes_nothing() {
    let tmp = TempDir::new().unwrap();
    let root = scaffold(&tmp);

    let tracked = root
        .join("documents")
        .join("specs")
        .join("AK-XX SDTM Specification v1.0.xlsx");
    fs::write(&tracked, b"edited by hand").unwrap();
    let before = file_names(&root.join("stats").join("final").join("macros"));

    let context = Context::build(raw_context(
        tmp.path(),
        Some(root.clone()),
        PathBuf::from(CONFIG),
    ))
    .unwrap();
    project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap()).unwrap();

    assert_eq!(fs::read(&tracked).unwrap(), b"edited by hand");
    assert_eq!(
        before,
        file_names(&root.join("stats").join("final").join("macros"))
    );
}

#[test]
fn reports_a_missing_reference_file() {
    let tmp = TempDir::new().unwrap();
    let broken = tmp.path().join("broken.json");
    fs::write(
        &broken,
        r#"{"projectRoot": "{{ project_root }}", "directories": [
            {"name": "out.xlsx", "directory": false, "reference": "/nonexistent/template.xlsx"}
        ]}"#,
    )
    .unwrap();

    let context = Context::build(raw_context(tmp.path(), None, broken)).unwrap();
    let result = project_scaffolding(ProjectScaffoldConfig::build(&context).unwrap());

    assert!(matches!(result, Err(ScaffoldError::ReferenceNotFound(_))));
}
```

- [ ] **Step 3: Run the integration tests**

Run:

```bash
cargo test -p project-scaffold --test scaffolding
```

Expected: 6 passed, 0 failed.

- [ ] **Step 4: Run the full suite for the crate and the workspace**

Run:

```bash
cargo test -p project-scaffold 2>&1 | tail -15
```

Expected: all unit and integration tests pass.

Then:

```bash
cargo test --workspace 2>&1 | tail -15
```

Expected: every crate's tests pass — this change must not break `als-resolver`, `terminology`,
`checklog`, `qc_result`, or `entities`.

- [ ] **Step 5: Add a crate README**

Create `crates/project-scaffold/README.md`. Note the four-backtick fence around the outer block so
the inner Rust fence survives:

````markdown
# project-scaffold

Materialises the directory scaffold of a clinical trial data processing project from a JSON
description.

## Usage

```rust
use project_scaffold::{Context, ProjectScaffoldConfig, RawContext, project_scaffolding};

let context = Context::build(RawContext {
    product: "ak".into(),
    trial: "xx".into(),
    purpose: "final".into(),
    root_path: "/projects".into(),
    project_root: None,
    study: None,
    config_path: "/etc/project-scaffold/config.json".into(),
})?;

let config = ProjectScaffoldConfig::build(&context)?;
project_scaffolding(config)?;
```

## Configuration

See [`.mock_data/project-scaffold/config.json`](../../.mock_data/project-scaffold/config.json)
for a worked example.

| Field | Meaning |
| --- | --- |
| `projectRoot` | where the tree is rooted; accepts `{{ project_root }}` |
| `directories[].name` | entry name; accepts `{{ product }}`, `{{ trial }}`, `{{ purpose }}`, `{{ study }}`, `{{ root_path }}`, `{{ project_root }}` |
| `directories[].directory` | `true` for a directory, `false` for a file |
| `directories[].children` | nested entries |
| `directories[].reference` | absolute path to a file copied to `name` |
| `directories[].migration.directory` | absolute path to a source directory |
| `directories[].migration.filter` | include-list of extensions, each with a leading dot |

Paths that already exist are never overwritten, so re-running scaffolding over an existing project
is a no-op.
````

- [ ] **Step 6: Commit**

```bash
git add .mock_data/project-scaffold/config.json crates/project-scaffold/tests/scaffolding.rs crates/project-scaffold/README.md
git commit -m "$(cat <<'EOF'
test(project-scaffold): cover scaffolding end to end against mock config

Co-Authored-By: Claude Code <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

**Spec coverage**

| Spec section | Task |
| --- | --- |
| §2 crate layout | Task 1 (manifest + files), Tasks 2–5 (one file each) |
| §3 Context + normalisation table | Task 3 |
| §3 `config_path` deviation | Task 3 (field), Task 8 (used) |
| §4 config model + `build()` | Task 4 |
| §5 template substitution | Task 2 (engine), Task 4 (applied to nodes) |
| §6 walk: reference / migration / directory / file / skip-if-exists | Tasks 5, 6, 7 |
| §7 errors | Task 1 (all variants), exercised in Tasks 3–8 |
| §8 mock data absolute paths | Task 8 Step 1 |
| §9 testing | Tasks 2–7 unit, Task 8 integration |

**Test count:** 7 template + 10 context + 7 config + 11 scaffold + 6 integration = 41.

**Placeholder scan:** no `TBD`, `TODO`, or "add appropriate error handling" steps; every code
change is shown in full.

**Type consistency:** `ScaffoldError::io`, `template::render`, `Context::build`,
`Context::template_vars`, `ProjectScaffoldConfig::build`, `scaffold::build_tree`, and the
`Node` / `Migration` field names are introduced once and used with identical signatures in every
later task.
