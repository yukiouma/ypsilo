use crate::{context::Context, Result};
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
