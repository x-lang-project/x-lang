use std::collections::{BTreeMap, BTreeSet};

use crate::{Applicability, Diagnostic, FileDatabase, SourceIdentifier, SpanEdit};

#[derive(Default)]
pub(super) struct EditPlan<'a> {
    pub(super) per_file: BTreeMap<SourceIdentifier, Vec<(usize, &'a SpanEdit)>>,
    pub(super) rejected: BTreeSet<usize>,
}

impl<'a> EditPlan<'a> {
    pub(super) fn prepare(database: &FileDatabase, diags: &'a [Diagnostic]) -> Self {
        let mut plan = Self::default();

        for (index, suggestion) in diags
            .iter()
            .flat_map(|diagnostic| &diagnostic.suggestions)
            .enumerate()
        {
            if suggestion.applicability != Applicability::MachineApplicable {
                continue;
            }

            for edit in &suggestion.edits {
                let valid = database
                    .get(edit.span.source)
                    .and_then(|file| file.source.get(edit.span.start..edit.span.end))
                    .is_some();

                if !valid {
                    plan.rejected.insert(index);
                }

                plan.per_file
                    .entry(edit.span.source)
                    .or_default()
                    .push((index, edit));
            }
        }

        plan.reject_overlaps();

        plan
    }

    fn reject_overlaps(&mut self) {
        for edits in self.per_file.values_mut() {
            edits.sort_by_key(|(_, edit)| edit.span.start);
            let overlaps = edits.windows(2).any(|pair| {
                matches!(pair, [(_, first), (_, second)] if first.span.end > second.span.start)
            });

            if overlaps {
                self.rejected.extend(edits.iter().map(|(index, _)| *index));
            }
        }
    }
}
