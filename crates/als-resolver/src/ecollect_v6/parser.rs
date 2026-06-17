use crate::AlsParseError;
use crate::ecollect_v6::context::EcollectParseContext;
use crate::ecollect_v6::{
    analytes, code_list, form_item, form_sets, forms, items, unit_groups, visits,
};
use crate::traits::AlsParser;
use entities::project::{Project, Visit};
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

        // Phase 5: Compute form ordinals based on visit order
        compute_form_ordinals(&mut context, &visit_list);

        // Sort forms by ordinal
        let mut forms: Vec<_> = context.forms.into_values().collect();
        forms.sort_by_key(|f| f.order);

        // Build and return Project
        Ok(Project {
            forms,
            visit: visit_list,
        })
    }
}

/// Compute form ordinals based on visit traversal order.
///
/// Algorithm:
/// 1. Sort visits by field order, initialize an empty form OID list
/// 2. Iterate visits in order, for each visit iterate its forms;
///    if form OID not in the list, push it
/// 3. Assign each form's ordinal to its index in the list
fn compute_form_ordinals(context: &mut EcollectParseContext, visits: &[Visit]) {
    // Sort visits by order field
    let mut sorted_visits = visits.to_vec();
    sorted_visits.sort_by_key(|v| v.order);

    // Build ordered form OID list
    let mut form_oid_list: Vec<String> = Vec::new();
    for visit in &sorted_visits {
        for form_oid in &visit.forms {
            if !form_oid_list.contains(form_oid) {
                form_oid_list.push(form_oid.clone());
            }
        }
    }

    // Assign ordinals to forms
    for form in context.forms.values_mut() {
        if let Some(index) = form_oid_list.iter().position(|oid| oid == &form.name) {
            form.order = index as i32 + 1;
        }
    }
}
