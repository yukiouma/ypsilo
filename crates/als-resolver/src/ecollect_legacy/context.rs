use entities::project::{CRFForm, ItemOption, Visit};
use std::collections::HashMap;

#[derive(Debug)]
pub struct LegacyParseContext {
    pub code_list_options: HashMap<String, Vec<ItemOption>>,
    pub analytes: HashMap<String, String>,
    pub forms: HashMap<String, CRFForm>,
    pub visits: HashMap<String, Visit>,
    pub event_form_bindings: HashMap<String, Vec<String>>,
}

impl Default for LegacyParseContext {
    fn default() -> Self {
        Self {
            code_list_options: HashMap::new(),
            analytes: HashMap::new(),
            forms: HashMap::new(),
            visits: HashMap::new(),
            event_form_bindings: HashMap::new(),
        }
    }
}

impl LegacyParseContext {
    pub fn new() -> Self {
        Self::default()
    }
}
