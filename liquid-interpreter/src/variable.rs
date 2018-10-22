use std::fmt;
use std::io::Write;

use error::{Error, Result, ResultLiquidChainExt};
use value::Path;
use value::Scalar;
use value::Value;

use super::Context;
use super::Expression;
use super::Renderable;

/// A `Value` reference.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variable {
    path: Vec<Expression>,
}

impl Variable {
    /// Create a `Value` reference.
    pub fn with_literal<S: Into<Scalar>>(value: S) -> Self {
        let expr = Expression::with_literal(value);
        let path = vec![expr];
        Self { path }
    }

    /// Append a literal.
    pub fn push_literal<S: Into<Scalar>>(mut self, value: S) -> Self {
        self.path.push(Expression::with_literal(value));
        self
    }

    /// Convert to a `Path`.
    pub fn evaluate(&self, context: &Context) -> Result<Value> {
        let path: Result<Path> = self
            .path
            .iter()
            .map(|e| e.evaluate(context))
            .map(|v| {
                let v = v?;
                let s = v
                    .as_scalar()
                    .ok_or_else(|| Error::with_msg(format!("Expected scalar, found `{}`", v)))?
                    .clone();
                Ok(s)
            }).collect();
        let mut path = path?;

        let value = match context.stack().get(&path) {
            Ok(value) => value.clone(),

            // If no value is found, it may still be a `.size` expression
            Err(err) => {
                let last = match path.pop() {
                    Some(v) => v,
                    None => return Err(err),
                };

                if &*last.to_str() == "size" {
                    let value = context.stack().get(&path)?;

                    match *value {
                        Value::Array(ref x) => Value::scalar(x.len() as i32),
                        Value::Object(ref x) => Value::scalar(x.len() as i32),
                        _ => return Err(err),
                    }
                } else {
                    return Err(err);
                }
            }
        };

        Ok(value)
    }
}

impl Extend<Scalar> for Variable {
    fn extend<T: IntoIterator<Item = Scalar>>(&mut self, iter: T) {
        let path = iter.into_iter().map(Expression::with_literal);
        self.path.extend(path);
    }
}

impl Extend<Expression> for Variable {
    fn extend<T: IntoIterator<Item = Expression>>(&mut self, iter: T) {
        let path = iter.into_iter();
        self.path.extend(path);
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.path.iter();
        let head = iter.next();
        match head {
            Some(head) => write!(f, "{}", head)?,
            None => return Ok(()),
        }
        for index in iter {
            write!(f, "[\"{}\"]", index)?;
        }
        Ok(())
    }
}

impl Renderable for Variable {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let value = self.evaluate(context)?;
        write!(writer, "{}", value).chain("Failed to render")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde_yaml;

    use super::super::ContextBuilder;
    use super::*;
    use value::Object;

    #[test]
    fn identifier_path_array_index() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test"]
"#,
        ).unwrap();
        let mut actual = Variable::with_literal("test_a");
        let index = vec![Scalar::new(0)];
        actual.extend(index);

        let mut context = ContextBuilder::new().set_globals(&globals).build();
        let actual = actual.render(&mut context).unwrap();
        assert_eq!(actual, "test".to_owned());
    }

    #[test]
    fn identifier_path_array_index_negative() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test1", "test2"]
"#,
        ).unwrap();
        let mut actual = Variable::with_literal("test_a");
        let index = vec![Scalar::new(-1)];
        actual.extend(index);

        let mut context = ContextBuilder::new().set_globals(&globals).build();
        let actual = actual.render(&mut context).unwrap();
        assert_eq!(actual, "test2".to_owned());
    }

    #[test]
    fn identifier_path_object() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a:
  - test_h: 5
"#,
        ).unwrap();
        let mut actual = Variable::with_literal("test_a");
        let index = vec![Scalar::new(0), Scalar::new("test_h")];
        actual.extend(index);

        let mut context = ContextBuilder::new().set_globals(&globals).build();
        let actual = actual.render(&mut context).unwrap();
        assert_eq!(actual, "5".to_owned());
    }
}
