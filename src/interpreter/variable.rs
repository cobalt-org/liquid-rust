use std::fmt;
use std::io::Write;

use itertools;

use error::{Result, ResultLiquidChainExt};
use value::Index;

use super::Context;
use super::Renderable;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variable {
    indexes: Vec<Index>,
}

impl Variable {
    pub fn new<I: Into<Index>>(value: I) -> Self {
        let indexes = vec![value.into()];
        Self { indexes }
    }

    pub fn indexes(&self) -> &[Index] {
        &self.indexes
    }
}

impl Extend<Index> for Variable {
    fn extend<T: IntoIterator<Item = Index>>(&mut self, iter: T) {
        self.indexes.extend(iter);
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = itertools::join(self.indexes().iter(), ".");
        write!(f, "{}", data)
    }
}

impl Renderable for Variable {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let value = context.stack().get_val_by_index(self.indexes.iter())?;
        write!(writer, "{}", value).chain("Failed to render")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde_yaml;

    use value::Object;
    use Parser;

    #[test]
    fn identifier_path_array_index() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test"]
"#,
        ).unwrap();
        let template = "array: {{ test_a[0] }}";

        let parser = Parser::new();
        let template = parser.parse(template).unwrap();
        let actual = template.render(&globals).unwrap();
        assert_eq!(actual, "array: test".to_owned());
    }

    #[test]
    fn identifier_path_array_index_negative() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test1", "test2"]
"#,
        ).unwrap();
        let template = "array: {{ test_a[-1] }}";

        let parser = Parser::new();
        let template = parser.parse(template).unwrap();
        let actual = template.render(&globals).unwrap();
        assert_eq!(actual, "array: test2".to_owned());
    }

    #[test]
    fn identifier_path_object_dot() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a:
  - test_h: 5
"#,
        ).unwrap();
        let template = "object_dot: {{ test_a[0].test_h }}\n";

        let parser = Parser::new();
        let template = parser.parse(template).unwrap();
        let actual = template.render(&globals).unwrap();
        assert_eq!(actual, "object_dot: 5\n".to_owned());
    }

    #[test]
    fn identifier_path_object_string() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a:
  - test_h: 5
"#,
        ).unwrap();
        let template = r#"object_string: {{ test_a[0]["test_h"] }}"#;

        let parser = Parser::new();
        let template = parser.parse(template).unwrap();
        let actual = template.render(&globals).unwrap();
        assert_eq!(actual, "object_string: 5".to_owned());
    }

    #[test]
    #[should_panic]
    fn identifier_path_subexpression() {
        let globals: Object = serde_yaml::from_str(
            r#"
somevar: test_h
test_a:
  - test_h: 5
"#,
        ).unwrap();
        let template = r#"result_string: {{ test_a[0][somevar] }}"#;

        let parser = Parser::new();
        let template = parser.parse(template).unwrap();
        let actual = template.render(&globals).unwrap();
        assert_eq!(actual, "result_string: 5".to_owned());
    }
}
