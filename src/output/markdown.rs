use tabled::{Table, Tabled, settings::Style};

#[allow(dead_code)]
pub fn render<T: Tabled>(value: &T) -> String {
    Table::new(std::iter::once(value))
        .with(Style::markdown())
        .to_string()
}

#[allow(dead_code)]
pub fn render_list<T: Tabled>(values: &[T]) -> String {
    Table::new(values)
        .with(Style::markdown())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Tabled)]
    struct TestRow {
        name: String,
        score: u32,
    }

    #[test]
    fn render_produces_markdown_table() {
        let row = TestRow {
            name: "hello".to_string(),
            score: 99,
        };
        let output = render(&row);
        assert!(output.contains("|"));
        assert!(output.contains("---"));
        assert!(output.contains("name"));
        assert!(output.contains("hello"));
        assert!(output.contains("99"));
    }

    #[test]
    fn render_list_produces_markdown_table() {
        let rows = vec![
            TestRow { name: "x".to_string(), score: 1 },
            TestRow { name: "y".to_string(), score: 2 },
        ];
        let output = render_list(&rows);
        assert!(output.contains("|"));
        assert!(output.contains("x"));
        assert!(output.contains("y"));
    }
}
