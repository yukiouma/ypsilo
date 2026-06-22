use crate::AlsParseError;
use crate::ecollect_legacy::context::LegacyParseContext;
use crate::ecollect_legacy::{analytes, code_list, events, event_form, forms, group_items};
use crate::traits::AlsParser;
use entities::project::{Project, Visit};
use std::path::Path;

pub struct EcollectLegacyParser;

impl AlsParser for EcollectLegacyParser {
    fn parse(&self, path: &Path) -> Result<Project, AlsParseError> {
        let mut context = LegacyParseContext::new();

        // Phase 1: Load reference data
        code_list::parse_code_list_items(path, &mut context)?;
        analytes::parse_analytes(path, &mut context)?;

        // Phase 2: Parse forms
        forms::parse_forms(path, &mut context)?;

        // Phase 3: Parse items (must happen after forms + reference data)
        group_items::parse_group_items(path, &mut context)?;

        // Phase 4: Parse visits
        events::parse_events(path, &mut context)?;

        // Phase 5: Link forms to visits via EventForm
        event_form::parse_event_form(path, &mut context)?;

        // Phase 6: Build final visit list with form bindings
        let visit_list = build_visits(&mut context);

        // Sort forms by ordinal
        let mut forms: Vec<_> = context.forms.into_values().collect();
        forms.sort_by_key(|f| f.order);

        Ok(Project {
            forms,
            visit: visit_list,
        })
    }
}

fn build_visits(context: &mut LegacyParseContext) -> Vec<Visit> {
    let mut sorted_visits: Vec<_> = context.visits.values_mut().collect();
    sorted_visits.sort_by_key(|v| v.order);

    // Step 1: Apply event_form_bindings to visits FIRST
    for visit in &mut sorted_visits {
        if let Some(form_oids) = context.event_form_bindings.get(&visit.code) {
            for form_oid in form_oids {
                if !visit.forms.contains(form_oid) {
                    visit.forms.push(form_oid.clone());
                }
            }
        }
    }

    // Step 2: Build ordered form OID list by iterating visits in order
    let mut form_oid_list: Vec<String> = Vec::new();
    for visit in &sorted_visits {
        for form_oid in &visit.forms {
            if !form_oid_list.contains(form_oid) {
                form_oid_list.push(form_oid.clone());
            }
        }
    }

    // Step 3: Assign ordinal to each form based on index in form_oid_list (1-based)
    for form in context.forms.values_mut() {
        if let Some(index) = form_oid_list.iter().position(|oid| oid == &form.name) {
            form.order = index as i32 + 1;
        }
    }

    sorted_visits
        .into_iter()
        .map(|v| Visit {
            code: v.code.clone(),
            name: v.name.clone(),
            order: v.order,
            forms: v.forms.clone(),
        })
        .collect()
}
