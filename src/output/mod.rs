pub mod json;
pub mod table;

use serde_json::Value;

pub fn render_value(value: &Value, json: bool, pretty: bool) -> String {
    if json {
        json::render(value, pretty)
    } else {
        table::render_value(value)
    }
}
