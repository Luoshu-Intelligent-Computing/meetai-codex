use std::collections::BTreeMap;

use crate::context::AdditionalContextDeveloperFragment;
use crate::context::AdditionalContextUserFragment;
use crate::context::ContextualUserFragment;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::protocol::AdditionalContextEntry;
use codex_protocol::protocol::AdditionalContextKind;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AdditionalContextStore {
    values: BTreeMap<String, AdditionalContextEntry>,
}

impl AdditionalContextStore {
    pub(crate) fn merge(
        &mut self,
        values: BTreeMap<String, AdditionalContextEntry>,
    ) -> Vec<ResponseInputItem> {
        let fragments = values
            .iter()
            .filter_map(|(key, entry)| {
                let projection = model_visible_projection(key, entry);
                let previous_projection = self
                    .values
                    .get(key)
                    .and_then(|entry| model_visible_projection(key, entry));
                (projection != previous_projection).then_some((key, projection))
            })
            .filter_map(|(key, entry)| entry.map(|entry| (key, entry)))
            .map(|(key, entry)| match entry.kind {
                AdditionalContextKind::Untrusted => {
                    AdditionalContextUserFragment::new(key.clone(), entry.value.clone())
                        .into_response_input_item()
                }
                AdditionalContextKind::Application => {
                    AdditionalContextDeveloperFragment::new(key.clone(), entry.value.clone())
                        .into_response_input_item()
                }
            })
            .collect();
        self.values = values;
        fragments
    }

    pub(crate) fn application_value(&self, key: &str) -> Option<&str> {
        self.values
            .get(key)
            .filter(|entry| entry.kind == AdditionalContextKind::Application)
            .map(|entry| entry.value.as_str())
    }
}

fn model_visible_projection(
    key: &str,
    entry: &AdditionalContextEntry,
) -> Option<AdditionalContextEntry> {
    if key != "application" || entry.kind != AdditionalContextKind::Application {
        return Some(entry.clone());
    }

    let Ok(Value::Object(mut application)) = serde_json::from_str(&entry.value) else {
        return Some(entry.clone());
    };
    if application.remove("library").is_none() {
        return Some(entry.clone());
    }

    (!application.is_empty()).then(|| AdditionalContextEntry {
        value: Value::Object(application).to_string(),
        kind: AdditionalContextKind::Application,
    })
}
