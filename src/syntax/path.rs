use error::{Error, Result};

use super::Context;
use super::Value;
use super::Renderable;
use super::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IdentifierPath {
    value: String,
    indexes: Vec<Value>,
}

impl Renderable for IdentifierPath {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let value = context
            .get_val(&self.value)
            .ok_or_else(|| Error::Render(format!("{} not found in context", self.value)))?
            .clone();

        let mut counter = self.indexes.len();
        let result = self.indexes.iter().fold(Ok(&value), |value, index| {
            // go through error
            let value = value?;
            counter -= 1;
            match (value, index) {
                (&Value::Array(ref value), &Value::Num(ref x)) => {
                    // at the first condition only is_normal is not enough
                    // because zero is not counted normal
                    if (*x != 0f32 && !x.is_normal()) ||
                        x.round() > (::std::isize::MAX as f32) ||
                            x.round() < (::std::isize::MIN as f32) {
                                return Error::renderer(&format!("bad array index: '{:?}'", x));
                            }

                    let idx = if *x >= 0f32 {
                        x.round() as usize
                    } else {
                        value.len() - (-x.round() as usize)
                    };
                    let err = ||
                        Error::Render(
                            format!("index out of range: got '{:?}' while array len is '{:?}'",
                                    idx,
                                    value.len()
                                   )
                            );
                    let value =
                        value
                        .get(idx)
                        .ok_or_else(err)?;
                    Ok(value)
                }
                (&Value::Array(_), x) => {
                    Error::renderer(
                        &format!("bad array index type: got array indexed by '{:?}'", x)
                    )
                }
                (&Value::Object(ref value), &Value::Str(ref x)) => {
                    let err = || Error::Render(format!("object element '{:?}' not found", x));
                    let value =
                        value
                        .get(x)
                        .ok_or_else(err)?;
                    Ok(value)
                }
                (&Value::Object(_), x) => {
                    Error::renderer(
                        &format!("bad object index type: expected string, but got '{:?}'", x)
                    )
                }
                (value, _) if counter == 0 => Ok(value),
                (value, _) => {
                    Error::renderer(
                        &format!("expected indexable element, but found '{:?}'", value)
                    )
                }
            }
        });

        result?.render(context)
    }
}

impl IdentifierPath {
    pub fn new(value: String) -> Self {
        Self {
            value: value,
            indexes: Vec::new(),
        }
    }
    pub fn append_indexes(&mut self, tokens: &[Token]) -> Result<()> {
        let rest = match tokens[0] {
            Token::Dot if tokens.len() > 1 => {
                match tokens[1] {
                    Token::Identifier(ref x) => self.indexes.push(Value::Str(x.clone())),
                    _ => {
                        return Error::parser("identifier", Some(&tokens[0]));
                    }
                };
                2
            }
            Token::OpenSquare if tokens.len() > 2 => {
                let index = match tokens[1] {
                    Token::StringLiteral(ref x) => Value::Str(x.clone()),
                    Token::NumberLiteral(ref x) => Value::Num(*x),
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

#[cfg(test)]
mod test {
    use serde_yaml;

    use syntax::Object;
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
