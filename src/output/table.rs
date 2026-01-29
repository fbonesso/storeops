use tabled::{Table, Tabled};

#[allow(dead_code)]
pub fn render<T: Tabled>(value: &T) -> String {
    Table::new(std::iter::once(value)).to_string()
}

#[allow(dead_code)]
pub fn render_list<T: Tabled>(values: &[T]) -> String {
    Table::new(values).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Tabled)]
    struct TestRow {
        name: String,
        value: i32,
    }

    #[test]
    fn render_produces_table_output() {
        let row = TestRow {
            name: "alpha".to_string(),
            value: 10,
        };
        let output = render(&row);
        assert!(output.contains("name"));
        assert!(output.contains("value"));
        assert!(output.contains("alpha"));
        assert!(output.contains("10"));
        assert!(output.contains("+") || output.contains("|") || output.contains("-"));
    }

    #[test]
    fn render_list_produces_table_with_multiple_rows() {
        let rows = vec![
            TestRow { name: "a".to_string(), value: 1 },
            TestRow { name: "b".to_string(), value: 2 },
        ];
        let output = render_list(&rows);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }
}
