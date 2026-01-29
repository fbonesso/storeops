use serde_json::Value;

pub fn render(value: &Value, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    } else {
        serde_json::to_string(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pretty_output_is_indented() {
        let val = json!({"name": "test", "count": 42});
        let output = render(&val, true);
        assert!(output.contains('\n'));
        assert!(output.contains("  "));
        assert!(output.contains("\"name\": \"test\""));
    }

    #[test]
    fn compact_output_has_no_newlines() {
        let val = json!({"name": "test", "count": 42});
        let output = render(&val, false);
        assert!(!output.contains('\n'));
    }

    #[test]
    fn renders_arrays() {
        let val = json!([1, 2, 3]);
        let output = render(&val, false);
        assert_eq!(output, "[1,2,3]");
    }
}
