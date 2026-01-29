pub mod json;
pub mod markdown;
pub mod table;

use crate::cli::OutputFormat;
use serde::Serialize;

#[allow(dead_code)]
pub fn render<T: Serialize + tabled::Tabled>(
    value: &T,
    format: OutputFormat,
    pretty: bool,
) -> String {
    match format {
        OutputFormat::Json => json::render(value, pretty),
        OutputFormat::Table => table::render(value),
        OutputFormat::Markdown => markdown::render(value),
    }
}

#[allow(dead_code)]
pub fn render_list<T: Serialize + tabled::Tabled>(
    values: &[T],
    format: OutputFormat,
    pretty: bool,
) -> String {
    match format {
        OutputFormat::Json => json::render(values, pretty),
        OutputFormat::Table => table::render_list(values),
        OutputFormat::Markdown => markdown::render_list(values),
    }
}

#[allow(dead_code)]
pub fn render_json<T: Serialize>(value: &T, pretty: bool) -> String {
    json::render(value, pretty)
}
