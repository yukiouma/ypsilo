use std::collections::HashMap;
use entities::project::{CRFForm, Visit, ItemOption};

/// Shared context during parsing. Accumulates data across phases.
pub struct ParseContext {
    /// DataDictionaries lookup: name -> Vec<DataDictionaryEntry>
    pub data_dictionary_entries: HashMap<String, Vec<DataDictionaryEntry>>,

    /// Parsed forms (OID -> CRFForm)
    pub forms: HashMap<String, CRFForm>,

    /// Parsed visits
    pub visits: Vec<Visit>,

    /// Raw form rows for later field assignment
    pub form_rows: Vec<FormRow>,
}

#[derive(Debug, Clone)]
pub struct DataDictionaryEntry {
    pub dictionary_name: String,
    pub coded_data: String,
    pub ordinal: i32,
    pub user_data_string: String,
    pub specify: bool,
}

#[derive(Debug, Clone)]
pub struct FormRow {
    pub oid: String,
    pub ordinal: i32,
    pub draft_form_name: String,
    pub link_folder_oid: Option<String>,
}

impl Default for ParseContext {
    fn default() -> Self {
        Self {
            data_dictionary_entries: HashMap::new(),
            forms: HashMap::new(),
            visits: Vec::new(),
            form_rows: Vec::new(),
        }
    }
}

impl ParseContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a DataDictionaryEntry to the lookup
    pub fn add_dictionary_entry(&mut self, entry: DataDictionaryEntry) {
        self.data_dictionary_entries
            .entry(entry.dictionary_name.clone())
            .or_default()
            .push(entry);
    }

    /// Get options for a DataDictionaryName
    pub fn get_options(&self, dictionary_name: &str) -> Vec<ItemOption> {
        self.data_dictionary_entries
            .get(dictionary_name)
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| ItemOption {
                        option_display: e.user_data_string.clone(),
                        annotations: Vec::new(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}