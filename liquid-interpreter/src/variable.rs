use std::fmt;
use std::io::Write;

use error::{Error, Result, ResultLiquidChainExt};
use value::Path;
use value::Scalar;

use super::Context;
use super::Expression;
use super::Renderable;

/// A `Value` reference.
#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    variable: Scalar,
    indexes: Vec<Expression>,
}

impl Variable {
    /// Create a `Value` reference.
    pub fn with_literal<S: Into<Scalar>>(value: S) -> Self {
        Self {
            variable: value.into(),
            indexes: Default::default(),
        }
    }

    /// Append a literal.
    pub fn push_literal<S: Into<Scalar>>(mut self, value: S) -> Self {
        self.indexes.push(Expression::with_literal(value));
        self
    }

    /// Convert to a `Path`.
    pub fn try_evaluate<'c>(&'c self, context: &'c Context) -> Option<Path<'c>> {
        let mut path = Path::with_index(self.variable.as_ref());
        path.reserve(self.indexes.len());
        for expr in &self.indexes {
            let v = expr.try_evaluate(context)?;
            let s = v.as_scalar()?.as_ref();
            path.push(s);
        }
        Some(path)
    }

    /// Convert to a `Path`.
    pub fn evaluate<'c>(&'c self, context: &'c Context) -> Result<Path<'c>> {
        let mut path = Path::with_index(self.variable.as_ref());
        path.reserve(self.indexes.len());
        for expr in &self.indexes {
            let v = expr.evaluate(context)?;
            let s = v
                .as_scalar()
                .ok_or_else(|| Error::with_msg(format!("Expected scalar, found `{}`", v)))?
                .as_ref();
            path.push(s);
        }
        Ok(path)
    }
}

impl Extend<Scalar> for Variable {
    fn extend<T: IntoIterator<Item = Scalar>>(&mut self, iter: T) {
        let path = iter.into_iter().map(Expression::with_literal);
        self.indexes.extend(path);
    }
}

impl Extend<Expression> for Variable {
    fn extend<T: IntoIterator<Item = Expression>>(&mut self, iter: T) {
        let path = iter.into_iter();
        self.indexes.extend(path);
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.variable)?;
        for index in self.indexes.iter() {
            write!(f, "[{}]", index)?;
        }
        Ok(())
    }
}

impl Renderable for Variable {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let path = self.evaluate(context)?;
        let value = context.stack().get(&path)?;
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
