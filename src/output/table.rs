use serde_json::{Map, Value};
use tabled::{
    builder::Builder,
    settings::{style::HorizontalLine, Span, Style},
};

const MAX_VALUE_LEN: usize = 60;

/// Keys that are noisy in JSON:API and should be dropped from table output.
const SKIP_KEYS: &[&str] = &["relationships", "links", "self"];

pub fn render_value(value: &Value) -> String {
    let inner = unwrap_data(value);

    match inner {
        Value::Array(arr) if !arr.is_empty() => build_array_table(None, arr),
        Value::Object(_) => render_single_object(inner),
        _ => format_cell(inner),
    }
}

/// Unwrap the `data` key from App Store Connect / JSON:API responses.
fn unwrap_data(value: &Value) -> &Value {
    value
        .as_object()
        .and_then(|obj| obj.get("data"))
        .unwrap_or(value)
}

/// Flatten a JSON:API-style object: inline `attributes` keys alongside `id`/`type`,
/// and drop noisy keys like `relationships` and `links`.
fn flatten_object(obj: &Map<String, Value>) -> Map<String, Value> {
    let mut flat = Map::new();
    for (key, val) in obj {
        if SKIP_KEYS.contains(&key.as_str()) {
            continue;
        }
        if key == "attributes" {
            if let Value::Object(attrs) = val {
                for (ak, av) in attrs {
                    flat.insert(ak.clone(), av.clone());
                }
                continue;
            }
        }
        flat.insert(key.clone(), val.clone());
    }
    flat
}

/// Normalize array items: flatten JSON:API resources if detected.
fn normalize_rows(arr: &[Value]) -> Vec<Map<String, Value>> {
    let is_jsonapi = arr.iter().any(|v| {
        v.as_object()
            .is_some_and(|o| matches!(o.get("attributes"), Some(Value::Object(_))))
    });

    arr.iter()
        .filter_map(|v| v.as_object())
        .map(|o| {
            if is_jsonapi {
                flatten_object(o)
            } else {
                o.clone()
            }
        })
        .collect()
}

fn collect_columns(rows: &[Map<String, Value>]) -> Vec<String> {
    let mut columns: Vec<String> = Vec::new();
    for row in rows {
        for key in row.keys() {
            if !columns.contains(key) {
                columns.push(key.clone());
            }
        }
    }
    columns
}

/// Build a table from an array of values, with an optional spanning title row.
fn build_array_table(title: Option<&str>, arr: &[Value]) -> String {
    let rows = normalize_rows(arr);

    if rows.is_empty() {
        let mut builder = Builder::default();
        builder.push_record(["Value"]);
        for item in arr {
            builder.push_record([format_cell(item)]);
        }
        return builder.build().with(Style::rounded()).to_string();
    }

    let columns = collect_columns(&rows);
    let col_count = columns.len().max(1);
    let mut builder = Builder::default();

    if let Some(t) = title {
        let mut title_row = vec![t.to_string()];
        title_row.resize(col_count, String::new());
        builder.push_record(title_row);
    }

    builder.push_record(&columns);

    for row in &rows {
        let cells: Vec<String> = columns
            .iter()
            .map(|col| row.get(col).map(format_cell).unwrap_or_default())
            .collect();
        builder.push_record(cells);
    }

    let mut table = builder.build();
    if title.is_some() {
        // Use rounded style with a custom line after the title that has no intersections
        let title_sep = HorizontalLine::filled('─').left('├').right('┤');
        let header_sep = HorizontalLine::filled('─')
            .left('├')
            .right('┤')
            .intersection('┼');
        table.with(
            Style::rounded()
                .top('─')
                .corner_top_left('╭')
                .corner_top_right('╮')
                .horizontals([(1, title_sep), (2, header_sep)]),
        );
        table.modify((0, 0), Span::column(col_count));
    } else {
        table.with(Style::rounded());
    }
    table.to_string()
}

fn render_single_object(value: &Value) -> String {
    let raw_obj = match value.as_object() {
        Some(o) => o,
        None => return format_cell(value),
    };

    let obj = if matches!(raw_obj.get("attributes"), Some(Value::Object(_))) {
        flatten_object(raw_obj)
    } else {
        raw_obj.clone()
    };

    let mut sections: Vec<String> = Vec::new();

    // Scalar fields as key-value table
    let scalar_fields: Vec<(&String, &Value)> = obj
        .iter()
        .filter(|(_, v)| !v.is_array() && !v.is_object())
        .collect();

    if !scalar_fields.is_empty() {
        let mut builder = Builder::default();
        builder.push_record(["Field", "Value"]);
        for (key, val) in &scalar_fields {
            builder.push_record([key.as_str(), &format_cell(val)]);
        }
        sections.push(builder.build().with(Style::rounded()).to_string());
    }

    // Nested arrays of objects as titled sub-tables
    for (key, val) in &obj {
        if let Value::Array(arr) = val {
            if !arr.is_empty() && arr.iter().any(|v| v.is_object()) {
                sections.push(build_array_table(Some(key), arr));
            }
        }
    }

    if sections.is_empty() {
        // Fallback: show everything as key-value
        let mut builder = Builder::default();
        builder.push_record(["Field", "Value"]);
        for (key, val) in &obj {
            builder.push_record([key.as_str(), &format_cell(val)]);
        }
        return builder.build().with(Style::rounded()).to_string();
    }

    sections.join("\n")
}

fn format_cell(value: &Value) -> String {
    let s = match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) => "[...]".to_string(),
        Value::Object(_) => "{...}".to_string(),
    };
    if s.len() > MAX_VALUE_LEN {
        format!("{}...", &s[..MAX_VALUE_LEN - 3])
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_array_of_objects() {
        let val = json!([
            {"name": "alpha", "value": 10},
            {"name": "beta", "value": 20}
        ]);
        let output = render_value(&val);
        assert!(output.contains("alpha"));
        assert!(output.contains("beta"));
        assert!(output.contains("name"));
        assert!(output.contains("value"));
    }

    #[test]
    fn render_single_object() {
        let val = json!({"status": "ok", "profile": "default"});
        let output = render_value(&val);
        assert!(output.contains("Field"));
        assert!(output.contains("Value"));
        assert!(output.contains("status"));
        assert!(output.contains("ok"));
    }

    #[test]
    fn render_unwraps_data_key() {
        let val = json!({"data": [{"id": "1", "type": "apps"}]});
        let output = render_value(&val);
        assert!(output.contains("id"));
        assert!(output.contains("apps"));
    }

    #[test]
    fn render_truncates_long_values() {
        let long = "a".repeat(100);
        let val = json!({"field": long});
        let output = render_value(&val);
        assert!(output.contains("..."));
    }

    #[test]
    fn render_object_only_nested_falls_back() {
        let val = json!({"nested": {"a": 1}});
        let output = render_value(&val);
        assert!(output.contains("nested"));
        assert!(output.contains("{...}"));
    }

    #[test]
    fn render_jsonapi_array_flattens_attributes() {
        let val = json!({"data": [
            {
                "type": "apps",
                "id": "123",
                "attributes": {"name": "MyApp", "bundleId": "com.example"},
                "relationships": {"builds": {"data": []}},
                "links": {"self": "https://..."}
            },
            {
                "type": "apps",
                "id": "456",
                "attributes": {"name": "Other", "bundleId": "com.other"},
                "relationships": {},
                "links": {}
            }
        ]});
        let output = render_value(&val);
        assert!(output.contains("MyApp"));
        assert!(output.contains("com.example"));
        assert!(output.contains("bundleId"));
        assert!(!output.contains("relationships"));
        assert!(!output.contains("links"));
    }

    #[test]
    fn render_jsonapi_single_object_flattens_attributes() {
        let val = json!({"data": {
            "type": "apps",
            "id": "123",
            "attributes": {"name": "MyApp", "bundleId": "com.example"},
            "relationships": {"builds": {"data": []}}
        }});
        let output = render_value(&val);
        assert!(output.contains("MyApp"));
        assert!(output.contains("com.example"));
        assert!(!output.contains("relationships"));
    }

    #[test]
    fn render_auth_status_with_profiles() {
        let val = json!({
            "active_profile": "google-default",
            "profiles": [
                {"name": "google-default", "store": "google", "active": true}
            ]
        });
        let output = render_value(&val);
        assert!(output.contains("active_profile"));
        assert!(output.contains("google-default"));
        assert!(output.contains("profiles"));
        assert!(output.contains("store"));
    }

    #[test]
    fn render_google_nested_objects() {
        let val = json!({
            "track": {"track": "production"},
            "commit": {"id": "abc123"}
        });
        let output = render_value(&val);
        assert!(output.contains("track"));
        assert!(output.contains("commit"));
    }
}
