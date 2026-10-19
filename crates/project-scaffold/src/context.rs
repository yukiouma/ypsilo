use crate::Result;
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
