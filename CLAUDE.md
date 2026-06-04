# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cargo workspace with no members yet. The src/main.rs file is orphaned (no package in the workspace to claim it).

## Build Commands

- `cargo build` — Build the project
- `cargo run` — Run the binary
- `cargo test` — Run tests
- `cargo check` — Type-check without building

## Important Rules

1. When adding dependencies, use `cargo add <crate>` instead of manually editing Cargo.toml — this ensures the latest version is installed.
2. When declaring modules, do not use `mod.rs` — use the 2024 edition's file-hierarchy style (e.g., `mod foo;` with `foo/mod.rs` becoming just `foo.rs`).
3. All crates in this workspace must use `edition = "2024"` and live under the `crates/` directory.
4. When adding common dependencies (e.g., `serde`, `thiserror`), add them to the workspace root `Cargo.toml` under `[workspace.dependencies]`, then have child crates reference them via the workspace — do not duplicate version definitions across crates.
