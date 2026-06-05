//! CRF entity types.

/// CRF Form — top-level container
#[derive(Debug)]
pub struct CRFForm {
    pub name: String,
    pub description: String,
    pub order: i32,
    pub items: Vec<CRFItem>,
    pub domains: Vec<Domain>,
    pub annotations: Vec<Annotation>,
}

/// CRF Item — a single form field
#[derive(Debug)]
pub struct CRFItem {
    pub name: String,
    pub label: String,
    pub item_option: Option<Vec<ItemOption>>,
    pub annotations: Vec<Annotation>,
    pub format: String,
    pub control_type: ControlType,
    pub item_unit: Option<ItemUnit>,
    pub not_variable: Option<bool>,
}

/// Control type enum
#[derive(Debug)]
pub enum ControlType {
    TEXT,
    SELECTION,
    CHECKBOX,
    DATETIME,
}

/// Item Option — selectable option within an item
#[derive(Debug)]
pub struct ItemOption {
    pub option_display: String,
    pub annotations: Vec<Annotation>,
}

/// Item Unit — unit label for an item
#[derive(Debug)]
pub struct ItemUnit {
    pub value: String,
    pub annotations: Vec<Annotation>,
}

/// Annotation — metadata attached to a form element
#[derive(Debug)]
pub struct Annotation {
    pub text: String,
    pub domain_name: Option<String>,
    pub not_submitted: bool,
    pub assign: bool,
}

/// Domain — SDTM domain reference
#[derive(Debug)]
pub struct Domain {
    pub name: String,
    pub description: String,
}

#[derive(Debug)]
pub struct Visit {
    pub code: String,
    pub name: String,
    pub order: i32,
    /// map to the oid field of struct CRFForm
    pub forms: Vec<String>,
}

#[derive(Debug)]
pub struct Project {
    pub forms: Vec<CRFForm>,
    pub visit: Vec<Visit>,
}
