use codex_api::SearchQuery;
use codex_api::SearchResponseLength;
use schemars::JsonSchema;
use schemars::r#gen::SchemaSettings;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
struct SearchQueryOnlyCommands {
    /// Query the internet search engine for a given list of queries.
    #[serde(skip_serializing_if = "Option::is_none")]
    search_query: Option<Vec<SearchQuery>>,
    /// Set the length of the response to be returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    response_length: Option<SearchResponseLength>,
}

pub(crate) fn commands_schema() -> Value {
    let schema = SchemaSettings::draft2019_09()
        .with(|settings| {
            settings.inline_subschemas = true;
            settings.option_add_null_type = false;
        })
        .into_generator()
        .into_root_schema_for::<SearchQueryOnlyCommands>();
    let schema = match serde_json::to_value(schema) {
        Ok(schema) => schema,
        Err(err) => panic!("search commands schema should serialize: {err}"),
    };
    let Value::Object(mut schema) = schema else {
        unreachable!("search commands schema must be an object");
    };

    let mut tool_schema = Map::new();
    for key in [
        "properties",
        "required",
        "type",
        "additionalProperties",
        "$defs",
        "definitions",
    ] {
        if let Some(value) = schema.remove(key) {
            tool_schema.insert(key.to_string(), value);
        }
    }
    Value::Object(tool_schema)
}

#[cfg(test)]
mod tests {
    use super::commands_schema;

    #[test]
    fn commands_schema_exposes_search_query_only() {
        let schema = commands_schema();
        let properties = schema["properties"]
            .as_object()
            .expect("schema properties should be an object");

        assert!(properties.contains_key("search_query"));
        assert!(properties.contains_key("response_length"));
        for unsupported in [
            "image_query",
            "open",
            "click",
            "find",
            "screenshot",
            "finance",
            "weather",
            "sports",
            "time",
        ] {
            assert!(
                !properties.contains_key(unsupported),
                "schema should not expose {unsupported}"
            );
        }
    }
}
