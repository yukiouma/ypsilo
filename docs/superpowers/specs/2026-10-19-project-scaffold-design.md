# Project Scaffold Crate — Design

- **Date:** 2026-10-19
- **Status:** Approved (brainstorming)
- **Revised:** 2026-10-20 — optional `purpose`, null on an unresolved template key, optional node name;
  2026-10-20 — `date` template variable, `uppercase` / `lowercase` methods on a placeholder
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
├── Cargo.toml          # chrono, serde, serde_json, thiserror (from workspace); tempfile as dev-dep
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
    pub purpose: Option<String>,    // unchanged, when given
    pub date: String,               // the current day, as yyyymmdd
    pub root_path: PathBuf,         // must exist
    pub project_root: PathBuf,      // explicit, or derived
    pub study: String,              // explicit, or derived
    pub config_path: PathBuf,       // must exist
}

impl Context {
    pub fn build(raw: RawContext) -> Result<Self>;
}
```

`RawContext` carries the same fields before normalisation, with `Option` where a value is derivable
or may be absent. It is generic over both its path type and its string type — `RawContext<P =
PathBuf, S = String>` — so callers may pass `&str`, `String`, `&Path`, or `PathBuf` in every field
without converting first. Both parameters default to the owning types, so existing callers are
unaffected. The validated `Context` owns concrete `PathBuf`s and `String`s, so no downstream
signature is generic.

`purpose` is optional: a study need not have one. `RawContext::new` takes only the required fields,
and `with_purpose`, `with_project_root` and `with_study` set the optional ones.

### Normalisation rules

| Field          | Rule                                                                        |
| -------------- | --------------------------------------------------------------------------- |
| `product`      | uppercased                                                                   |
| `trial`        | uppercased                                                                   |
| `purpose`      | unchanged, and absent when not given                                         |
| `date`         | the current day, as `yyyymmdd` — set by `Context::build`, not an input       |
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
    #[serde(default)]
    pub name: Option<String>,
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
corresponding context value. Recognised keys: `product`, `trial`, `purpose`, `date`, `root_path`,
`project_root`, `study`. `purpose` is only recognised when the context carries one.

A placeholder may name a **method** to apply to the value it resolves to, after a `|`:
`{{ study | uppercase }}` splits into the key `study` and the method `uppercase`. The known
methods are `uppercase` and `lowercase`; any other method is **ignored**, and the value is used as
it stands — an unknown method never fails a config. The method applies to the whole value, so
`{{ study | lowercase }} Data Issue Log.xlsx` reads as `ak-xx Data Issue Log.xlsx`.

A key the context does not have leaves the field it appears in **null** rather than half-rendered:
a name of `"{{ purpose }}"` on a study without a purpose resolves to nothing, and so does
`"{{ study }} {{ purpose }}.xlsx"` — the whole name is null, not just the one placeholder. The
template is either fully resolved or it is not.

`project_root` is the exception: it has no null to fall back to, so an unresolved key there is the
`UnknownTemplateKey` error. The two behaviours live side by side in `template.rs` — `render`
resolves leniently and yields nothing when a key is missing, `render_required` fails.

A hand-rolled scan is used rather than a `mustache`/`handlebars` dependency: the config only ever
uses bare `{{ identifier }}` lookups against a fixed key set, so a full template engine would add a
dependency and an escaping model the crate does not need.

## 6. Scaffolding walk

`project_scaffolding(config)` walks `config.directories` recursively from the resolved
`project_root`:

- **Node without a name** — skipped, along with its children. A name is absent when the config
  omits it, or when a placeholder in it had no value to resolve to.
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
| `UnknownTemplateKey`    | a required `{{ ... }}` names a key the context does not define |

## 8. Mock data

`.mock_data/project-scaffold/config.json` is updated so that every `reference` and
`migration.directory` is an absolute path under
`/root/coding/project/ypsilo/.mock_data/project-scaffold/references/`.

Consequence: the fixture becomes machine-specific and will not resolve on another checkout. This was
chosen in preference to resolving those paths at runtime against a configurable base.

## 9. Testing

Unit tests:

- template substitution of each recognised key, embedded within surrounding text, the null result
  for an unknown key, and the `UnknownTemplateKey` error from the strict variant;
- the `uppercase` and `lowercase` methods, an ignored unknown method, and a method on a key that
  does not exist;
- `Context::build` setting `date` to the current day as `yyyymmdd`;
- `Context::build` normalisation: uppercasing, `project_root` derivation, `study` derivation, the
  optional `purpose`, and each existence-check error;
- `Node::resolve` leaving a name null when the config omits it or its key is unresolvable;
- the scaffolding walk skipping a node without a name, and its children with it.

Integration tests (over `tempfile`):

- scaffolding the full `config.json` tree produces the expected directory set;
- `reference` files are copied under their configured (templated) names;
- `migration` copies the `.sas` files and **not** the `.xlsx` files;
- a second run over the same directory is a no-op and does not overwrite;
- a missing reference file produces `ReferenceNotFound`;
- a context without a purpose leaves the `{{ purpose }}` branch out of the tree.
