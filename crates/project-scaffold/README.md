# project-scaffold

Materialises the directory scaffold of a clinical trial data processing project from a JSON
description.

## Usage

```rust
use project_scaffold::{Context, ProjectScaffoldConfig, RawContext, project_scaffolding};

let context = Context::build(RawContext {
    product: "ak",
    trial: "xx",
    purpose: "final",
    root_path: "/projects",
    project_root: None,
    study: None,
    config_path: "/etc/project-scaffold/config.json",
})?;

let config = ProjectScaffoldConfig::build(&context)?;
project_scaffolding(config)?;
```

`RawContext` is generic over both its path type and its string type, so `&str`, `String`, `&Path`,
and `PathBuf` all work in every field without conversion. Both parameters default to the owning
types, so `PathBuf`/`String` callers are unaffected. The validated `Context` it returns owns plain
`PathBuf`s and `String`s.

`Context::build` normalises its input: `product` and `trial` are uppercased, `study` defaults to
`{PRODUCT}-{TRIAL}`, and `project_root` defaults to `{root_path}/{product}/{trial}` in lower case.
`root_path`, an explicit `project_root`, and `config_path` must already exist.

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

`reference` and `migration.directory` are absolute and are checked for existence before
scaffolding; a missing one is an error naming the path. A migration copies the immediate children
of its source directory whose extension is in `filter` — it does not recurse.

Paths that already exist are never overwritten, so re-running scaffolding over an existing project
is a no-op.
