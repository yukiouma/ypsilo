# Project Scaffold Crate — Design

- **Date:** 2026-10-19
- **Status:** Approved (brainstorming)
- **Owner:** ypsilo / project-scaffold

## 1. Goal

Implement the `project-scaffold` crate at `crates/project-scaffold/`: a library that materialises the
directory scaffold of a clinical trial data processing project from a JSON configuration.

The entry point is:

```rust
pub fn project_scaffolding(config: ProjectScaffoldConfig) -> Result<()>
```

`ProjectScaffoldConfig` is built from a `Context` that carries the per-study identifiers
(product, trial, purpose, root path, project root, study) plus the path to the config JSON.

## 2. Crate layout

```
crates/project-scaffold/
├── Cargo.toml          # serde, serde_json, thiserror (from workspace); tempfile as dev-dep
├── src/
│   ├── lib.rs          # project_scaffolding(), Result alias, re-exports
│   ├── context.rs      # Context / RawContext + normalisation + validation
│   ├── config.rs       # ProjectScaffoldConfig, Node, Migration
│   ├── template.rs     # {{ var }} substitution
│   ├── scaffold.rs     # recursive walk: mkdir / copy / skip
│   └── error.rs        # ScaffoldError
└── tests/
    └── scaffolding.rs  # integration tests over tempfile
```

Follows the workspace rules: `edition = "2024"`, no `mod.rs` (file-hierarchy modules), shared
dependencies referenced from `[workspace.dependencies]`.

## 3. Context

```rust
pub struct Context {
    pub product: String,            // uppercased
    pub trial: String,              // uppercased
    pub purpose: String,            // unchanged
    pub root_path: PathBuf,         // must exist
    pub project_root: Option<PathBuf>,
    pub study: Option<String>,
    pub config_path: PathBuf,       // must exist
}

impl Context {
    pub fn build(raw: RawContext) -> Result<Self>;
}
```

`RawContext` carries the same fields before normalisation, with `Option` where a value is derivable.
It is generic over its path type — `RawContext<P = PathBuf>` — so callers may pass `&str`, `String`,
`&Path`, or `PathBuf` for `root_path`, `project_root`, and `config_path` without converting first.
The validated `Context` owns concrete `PathBuf`s, so no downstream signature is generic.

### Normalisation rules

| Field          | Rule                                                                        |
| -------------- | --------------------------------------------------------------------------- |
| `product`      | uppercased                                                                   |
| `trial`        | uppercased                                                                   |
| `purpose`      | unchanged                                                                    |
| `root_path`    | must exist, else error                                                       |
| `project_root` | if `Some`, must exist; if `None`, derive `{root_path}/{product}/{trial}`     |
|                | using the **lower-cased** product and trial                                  |
| `study`        | if `None`, derive `{PRODUCT}-{TRIAL}` (upper-cased)                         |
| `config_path`  | must exist, else error                                                       |

### Deviation from the original requirements

The stated `Context` field list did not include a config path, but the configuration is loaded from
a JSON file. `config_path` is added as a seventh field, which keeps the required signature
`ProjectScaffoldConfig::build(context: Context)` intact — the path rides along on the context
rather than becoming an extra parameter.

## 4. Configuration model

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScaffoldConfig {
    pub project_root: String,       // "projectRoot"; may contain {{ project_root }}
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
```

`reference` and `migration.directory` are **absolute** paths. The crate validates that each one
exists and errors naming the offending path.

`ProjectScaffoldConfig::build(context: Context) -> Result<ProjectScaffoldConfig>` reads the JSON at
`context.config_path`, deserialises it, and resolves template placeholders in `project_root` and in
every node `name`.

## 5. Template substitution

Every `{{ key }}` occurrence in a node `name` (and in `project_root`) is replaced with the
corresponding context value. Recognised keys: `product`, `trial`, `purpose`, `root_path`,
`project_root`, `study`. An unknown key is an error rather than a silent empty string.

A hand-rolled scan is used rather than a `mustache`/`handlebars` dependency: the config only ever
uses bare `{{ identifier }}` lookups against a fixed key set, so a full template engine would add a
dependency and an escaping model the crate does not need.

## 6. Scaffolding walk

`project_scaffolding(config)` walks `config.directories` recursively from the resolved
`project_root`:

- **Node with `reference`** — verify the source exists, then copy it to `base/{name}`.
- **Node with `migration`** — create the directory, verify the source directory exists, then copy
  every entry in it whose **extension** is in `filter`. The traversal is **non-recursive**: only
  the immediate children of the source directory are considered.
- **Node with `directory: true`** — create the directory, then recurse into `children`.
- **Node with `directory: false`** and no `reference` — create an empty file.

In every case, **if the target path already exists, it is skipped** — existing directories and
files are never overwritten, and re-running scaffolding over an already-scaffolded project is a
no-op.

`filter` is an include-list matched on file extension: `filter: [".sas"]` copies the 45 `.sas` files
in `general-marcos` and skips its two `.xlsx` files.

Errors fail fast and carry the path being constructed, so a failure deep in the tree is
identifiable.

## 7. Errors

`ScaffoldError` (thiserror):

| Variant                 | Meaning                                            |
| ----------------------- | -------------------------------------------------- |
| `Io`                    | underlying I/O failure, wrapped with `#[from]`     |
| `ConfigRead`            | config JSON could not be read                       |
| `ConfigParse`           | config JSON was malformed                           |
| `RootPathNotFound`      | `root_path` does not exist                          |
| `ProjectRootNotFound`   | explicit `project_root` does not exist              |
| `ConfigPathNotFound`    | `config_path` does not exist                        |
| `ReferenceNotFound`     | a node's `reference` source does not exist          |
| `MigrationDirNotFound`  | a node's `migration.directory` does not exist       |
| `UnknownTemplateKey`    | `{{ ... }}` names a key the context does not define |

## 8. Mock data

`.mock_data/project-scaffold/config.json` is updated so that every `reference` and
`migration.directory` is an absolute path under
`/root/coding/project/ypsilo/.mock_data/project-scaffold/references/`.

Consequence: the fixture becomes machine-specific and will not resolve on another checkout. This was
chosen in preference to resolving those paths at runtime against a configurable base.

## 9. Testing

Unit tests:

- template substitution of each recognised key, embedded within surrounding text, and the
  unknown-key error;
- `Context::build` normalisation: uppercasing, `project_root` derivation, `study` derivation, and
  each existence-check error.

Integration tests (over `tempfile`):

- scaffolding the full `config.json` tree produces the expected directory set;
- `reference` files are copied under their configured (templated) names;
- `migration` copies the `.sas` files and **not** the `.xlsx` files;
- a second run over the same directory is a no-op and does not overwrite;
- a missing reference file produces `ReferenceNotFound`.
