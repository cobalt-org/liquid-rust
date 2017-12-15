use error::{Error, Result};
use value::Index;

use super::Context;
use super::Renderable;
use super::Token;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    indexes: Vec<Index>,
}

fn coerce(index: f32) -> Option<isize> {
    // at the first condition only is_normal is not enough
    // because zero is not counted normal
    if (index != 0f32 && !index.is_normal()) || index.round() > (::std::isize::MAX as f32) ||
       index.round() < (::std::isize::MIN as f32) {
        None
    } else {
        Some(index.round() as isize)
    }
}

impl Variable {
    pub fn new<I: Into<Index>>(value: I) -> Self {
        let indexes = vec![value.into()];
        Self { indexes }
    }

    pub fn indexes(&self) -> &[Index] {
        &self.indexes
    }

    pub fn append_indexes(&mut self, tokens: &[Token]) -> Result<()> {
        let rest = match tokens[0] {
            Token::Dot if tokens.len() > 1 => {
                match tokens[1] {
                    Token::Identifier(ref x) => self.indexes.push(Index::with_key(x.as_ref())),
                    _ => {
                        return Error::parser("identifier", Some(&tokens[0]));
                    }
                };
                2
            }
            Token::OpenSquare if tokens.len() > 2 => {
                let index = match tokens[1] {
                    Token::StringLiteral(ref x) => Index::with_key(x.as_ref()),
                    Token::NumberLiteral(ref x) => {
                        let x = coerce(*x)
                            .ok_or_else(|| Error::Parser(format!("Invalid index {}", x)))?;
                        Index::with_index(x)
                    }
                    _ => {
                        return Error::parser("number | string", Some(&tokens[0]));
                    }
                };
                self.indexes.push(index);

                if tokens[2] != Token::CloseSquare {
                    return Error::parser("]", Some(&tokens[1]));
                }
                3
            }
            _ => return Ok(()),
        };

        if tokens.len() > rest {
            self.append_indexes(&tokens[rest..])
        } else {
            Ok(())
        }
    }
}

impl Renderable for Variable {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let value = context.get_val_by_index(self.indexes.iter())?;
        Ok(Some(value.to_string()))
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
