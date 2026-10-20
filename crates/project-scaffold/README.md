# project-scaffold

Materialises the directory scaffold of a clinical trial data processing project from a JSON
description.

## Usage

```rust
use project_scaffold::{Context, ProjectScaffoldConfig, RawContext, project_scaffolding};

let context = Context::build(RawContext {
    product: "ak",
    trial: "xx",
    purpose: Some("final"),
    root_path: "/projects",
    config_path: "/etc/project-scaffold/config.json",
    ..Default::default()
})?;

let config = ProjectScaffoldConfig::build(&context)?;
project_scaffolding(config)?;
```

`purpose`, `project_root` and `study` are optional. Omit them with `..Default::default()` as above,
or use the constructor and fluent setters:

```rust
let context = Context::build(
    RawContext::new("ak", "xx", "/projects", "/etc/project-scaffold/config.json")
        .with_purpose("final")
        .with_study("AK-999"),
)?;
```

`RawContext` is generic over both its path type and its string type, so `&str`, `String`, `&Path`,
and `PathBuf` all work in every field without conversion. Both parameters default to the owning
types, so `PathBuf`/`String` callers are unaffected. The validated `Context` it returns owns plain
`PathBuf`s and `String`s.

`Context::build` normalises its input: `product` and `trial` are uppercased, `study` defaults to
`{PRODUCT}-{TRIAL}`, and `project_root` defaults to `{root_path}/{product}/{trial}` in lower case.
`root_path`, an explicit `project_root`, and `config_path` must already exist. `purpose` is kept as
given, and stays absent when the context carries none.

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

`{{ purpose }}` is only available when the context carries a purpose. A name that names a key the
context does not have — `{{ purpose }}` without a purpose, say — is left unresolved, and so is any
name that mixes it with other text: the whole name becomes null and the entry is skipped. A name
omitted from the config is skipped the same way. `projectRoot` has no such luxury: an unresolved
key there is an error, since a scaffold needs a root to build into.

`reference` and `migration.directory` are absolute and are checked for existence before
scaffolding; a missing one is an error naming the path. A migration copies the immediate children
of its source directory whose extension is in `filter` — it does not recurse.

Copied files keep the access and modification times of their source, so a template's age survives
the copy instead of being stamped with the moment of scaffolding. The creation time of a copy is
always the moment of the copy — no platform-independent way exists to set it.

Paths that already exist are never overwritten, so re-running scaffolding over an existing project
is a no-op.
