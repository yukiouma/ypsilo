use crate::ecollect_v6::context::EcollectParseContext;
use crate::ecollect_v6::{code_list, analytes, form_sets, forms, items, form_item, unit_groups, visits};
use crate::traits::AlsParser;
use crate::AlsParseError;
use entities::project::Project;
use std::path::Path;

/// Ecollect v6 ALS parser implementation.
pub struct EcollectV6Parser;

impl AlsParser for EcollectV6Parser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let mut context = EcollectParseContext::new();

        // Phase 1: Load reference data
        code_list::parse_code_list_items(path, &mut context)?;
        analytes::parse_analytes(path, &mut context)?;
        form_sets::parse_form_sets(path, &mut context)?;
        unit_groups::parse_unit_groups(path, &mut context)?;

        // Phase 2: Parse forms
        forms::parse_forms(path, &mut context)?;

        // Phase 3: Parse items and form-item linkage
        items::parse_items(path, &mut context)?;
        form_item::parse_form_item(path, &mut context)?;

        // Phase 4: Parse visits
        let visit_list = visits::parse_visits(path, &mut context)?;

        // Build and return Project
        Ok(Project {
            forms: context.forms.into_values().collect(),
            visit: visit_list,
        })
    }
}
