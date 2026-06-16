use entities::project::{CRFForm, ItemOption, Visit};
use std::collections::HashMap;

/// Internal item definition from Items worksheet.
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub oid: String,
    pub item_name: String,
    pub sas_field_name: String,
    pub control_type: String,
    pub data_format: String,
    pub code_list_oid: Option<String>,
    pub unit_group_oid: Option<String>,
}

/// Shared context during ecollect v6 parsing. Accumulates data across phases.
pub struct EcollectParseContext {
    /// CodeListOID → Vec<ItemOption>
    pub code_list_options: HashMap<String, Vec<ItemOption>>,
    /// AnalyteCode → AnalyteName (from AnalytesInTheStudy)
    pub analytes: HashMap<String, String>,
    /// FormsetOID → FormsetName (from FormSets sheet, for visit name lookup)
    pub formset_names: HashMap<String, String>,
    /// Parsed forms (FormOID → CRFForm)
    pub forms: HashMap<String, CRFForm>,
    /// Item definitions (ItemOID → ItemDef)
    pub item_definitions: HashMap<String, ItemDef>,
    /// Visit code → Vec<FormOID> bindings discovered from Plan* sheets
    pub visit_form_bindings: HashMap<String, Vec<String>>,
    /// UnitGroupOID → Vec<UnitName> (for item_unit resolution)
    pub unit_groups: HashMap<String, Vec<String>>,
}

impl Default for EcollectParseContext {
    fn default() -> Self {
        Self {
            code_list_options: HashMap::new(),
            analytes: HashMap::new(),
            formset_names: HashMap::new(),
            forms: HashMap::new(),
            item_definitions: HashMap::new(),
            visit_form_bindings: HashMap::new(),
            unit_groups: HashMap::new(),
        }
    }
}

impl EcollectParseContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Split compound OID (e.g., "YN=[1|是]") on first "=" and return the key part.
    pub fn split_oid(oid: &str) -> &str {
        oid.splitn(2, '=').next().unwrap_or(oid)
    }
}
