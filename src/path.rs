use Renderable;
use value::Value;
use context::Context;
use token::Token;
use token::Token::*;
use error::{Error, Result};

#[derive(Debug)]
pub struct IdentifierPath {
    value: String,
    indexes: Vec<Value>,
}

impl Renderable for IdentifierPath {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        println!("{:?}", self);
        let value = context
            .get_val(&self.value)
            .ok_or(Error::Render(
                format!("{} not found in context", self.value),
            ))?
            .clone();

        let mut counter = self.indexes.len();
        let result = self.indexes
            .iter()
            .fold(Ok(&value), |value, index| {
                // go through error
                let value = if let Ok(ref value) = value {
                    value
                } else {
                    return value
                };
                counter-=1;
       match (value, index) {
            (&&Value::Array(ref value), &Value::Num(ref x)) => {
                if (*x != 0f32 && !x.is_normal()) // only is_normal is not enough, because zero is not counted normal
                    || *x < 0f32 ||
                    x.round() > (::std::usize::MAX as f32)
                {
                    return Error::renderer("bad array index");
                }
                let idx = x.round() as usize;
                let value = value.get(idx).ok_or(Error::Render(
                    "index out of range".to_string(),
                ))?;
                Ok(&value)
            }
            (&&Value::Array(_), _) => Error::renderer("bad array index type"),
            (&&Value::Object(ref value), &Value::Str(ref x)) => {
                let value = value.get(x).ok_or(Error::Render(
                    "object element not found".to_string(),
                ))?;
                Ok(&value)
            }
            (&&Value::Object(_), _) => Error::renderer("bad object index"),
            (&value, _) if counter == 0  => Ok(value),
            _ => Error::renderer("non-indexable element found while indexable expected")
        }
        });

        match result {
            Ok(result) => result.render(context),
            Err(e) => Error::renderer(&format!("rendering error: {}", e)),
        }
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
            Dot if tokens.len() > 1 => {
                match tokens[1] {
                    Identifier(ref x) => self.indexes.push(Value::Str(x.clone())),
                    _ => {
                        return Error::parser("identifier", Some(&tokens[0]));
                    }
                };
                2
            }
            OpenSquare if tokens.len() > 2 => {
                let index = match tokens[1] {
                    StringLiteral(ref x) => Value::Str(x.clone()),
                    NumberLiteral(ref x) => Value::Num(*x),
                    _ => {
                        return Error::parser("number | string", Some(&tokens[0]));
                    }
                };
                self.indexes.push(index);

                if tokens[2] != CloseSquare {
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
    use value::Value;
    use parse;
    use Renderable;
    use Context;
    use LiquidOptions;
    use std::collections::HashMap;

    #[test]
    fn identifier_path() {
        let options = LiquidOptions::with_known_blocks();
        let template = concat!(
            "array: {{ test_a[0] }}\n",
            "complex_dot: {{ test_a[0].test_h }}\n",
            "complex_string: {{ test_a[0][\"test_h\"] }}\n"
        );

        let mut context = Context::new();
        let mut internal = HashMap::new();
        internal.insert("test_h".to_string(), Value::Num(5f32));

        let test = Value::Array(vec![Value::Object(internal)]);
        context.set_val("test_a", test);

        let template = parse(template, options).unwrap();
        assert_eq!(
            template.render(&mut context).unwrap(),
            Some(
                concat!(
                    "array: test_h: 5\n",
                    "complex_dot: 5\n",
                    "complex_string: 5\n"
                ).to_owned(),
            )
        );
    }
}
