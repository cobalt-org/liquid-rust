use std::fmt;

use crate::error::{Error, Result};
use crate::model::Path;
use crate::model::Scalar;
use crate::model::{ValueCow, ValueView};

use super::Expression;
use super::Runtime;

/// A `Value` reference.
#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    variable: VariableRoot,
    indexes: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
enum VariableRoot {
    Identifier(Scalar),
    Expression(Box<Expression>),
}

impl Variable {
    /// Create a `Value` reference.
    pub fn with_literal<S: Into<Scalar>>(value: S) -> Self {
        Self {
            variable: VariableRoot::Identifier(value.into()),
            indexes: Default::default(),
        }
    }

    /// Create a `Value` reference from an expression root.
    pub fn with_expression(expression: Expression) -> Self {
        Self {
            variable: VariableRoot::Expression(Box::new(expression)),
            indexes: Default::default(),
        }
    }

    /// Append a literal.
    pub fn push_literal<S: Into<Scalar>>(mut self, value: S) -> Self {
        self.indexes.push(Expression::with_literal(value));
        self
    }

    /// Convert to a `Path`.
    pub fn try_evaluate<'c>(&'c self, runtime: &'c dyn Runtime) -> Option<Path<'c>> {
        let mut path = Path::with_index(self.try_evaluate_root(runtime)?);
        path.reserve(self.indexes.len());
        for expr in &self.indexes {
            let v = expr.try_evaluate(runtime)?;
            let s = match v {
                ValueCow::Owned(v) => v.into_scalar(),
                ValueCow::Borrowed(v) => v.as_scalar(),
            }?;
            path.push(s);
        }
        Some(path)
    }

    /// Convert to a `Path`.
    pub fn evaluate<'c>(&'c self, runtime: &'c dyn Runtime) -> Result<Path<'c>> {
        let mut path = Path::with_index(self.evaluate_root(runtime)?);
        path.reserve(self.indexes.len());
        for expr in &self.indexes {
            let v = expr.evaluate(runtime)?;
            let s = match v {
                ValueCow::Owned(v) => v.into_scalar(),
                ValueCow::Borrowed(v) => v.as_scalar(),
            }
            .ok_or_else(|| {
                let v = expr.evaluate(runtime).expect("lookup already verified");
                let v = v.source();
                let msg = format!("Expected scalar, found `{}`", v);
                Error::with_msg(msg)
            })?;
            path.push(s);
        }
        Ok(path)
    }

    fn try_evaluate_root<'c>(&'c self, runtime: &'c dyn Runtime) -> Option<Scalar> {
        match &self.variable {
            VariableRoot::Identifier(value) => Some(value.clone()),
            VariableRoot::Expression(expression) => {
                let value = expression.try_evaluate(runtime)?;
                match value {
                    ValueCow::Owned(value) => value.into_scalar(),
                    ValueCow::Borrowed(value) => value.as_scalar().map(|value| value.into_owned()),
                }
            }
        }
    }

    fn evaluate_root<'c>(&'c self, runtime: &'c dyn Runtime) -> Result<Scalar> {
        match &self.variable {
            VariableRoot::Identifier(value) => Ok(value.clone()),
            VariableRoot::Expression(expression) => {
                let value = expression.evaluate(runtime)?;
                match value {
                    ValueCow::Owned(value) => {
                        let rendered = value.source().to_string();
                        value
                            .into_scalar()
                            .ok_or_else(|| Error::with_msg(format!("Expected scalar, found `{}`", rendered)))
                    }
                    ValueCow::Borrowed(value) => value.as_scalar().map(|value| value.into_owned()).ok_or_else(|| {
                        let rendered = value.source();
                        Error::with_msg(format!("Expected scalar, found `{}`", rendered))
                    }),
                }
            }
        }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variable {
            VariableRoot::Identifier(value) => write!(f, "{}", value.render())?,
            VariableRoot::Expression(expression) => write!(f, "[{}]", expression)?,
        }
        for index in self.indexes.iter() {
            write!(f, "[{}]", index)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::Object;
    use crate::model::ValueViewCmp;

    use super::super::RuntimeBuilder;
    use super::super::StackFrame;

    #[test]
    fn identifier_path_array_index() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test"]
"#,
        )
        .unwrap();
        let mut var = Variable::with_literal("test_a");
        let index = vec![Scalar::new(0)];
        var.extend(index);

        let runtime = RuntimeBuilder::new().build();
        let runtime = StackFrame::new(&runtime, &globals);
        let actual = var.evaluate(&runtime).unwrap();
        let actual = runtime.get(&actual).unwrap();
        assert_eq!(actual, ValueViewCmp::new(&"test"));
    }

    #[test]
    fn identifier_path_array_index_negative() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a: ["test1", "test2"]
"#,
        )
        .unwrap();
        let mut var = Variable::with_literal("test_a");
        let index = vec![Scalar::new(-1)];
        var.extend(index);

        let runtime = RuntimeBuilder::new().build();
        let runtime = StackFrame::new(&runtime, &globals);
        let actual = var.evaluate(&runtime).unwrap();
        let actual = runtime.get(&actual).unwrap();
        assert_eq!(actual, ValueViewCmp::new(&"test2"));
    }

    #[test]
    fn identifier_path_object() {
        let globals: Object = serde_yaml::from_str(
            r#"
test_a:
  - test_h: 5
"#,
        )
        .unwrap();
        let mut var = Variable::with_literal("test_a");
        let index = vec![Scalar::new(0), Scalar::new("test_h")];
        var.extend(index);

        let runtime = RuntimeBuilder::new().build();
        let runtime = StackFrame::new(&runtime, &globals);
        let actual = var.evaluate(&runtime).unwrap();
        let actual = runtime.get(&actual).unwrap();
        assert_eq!(actual, ValueViewCmp::new(&5));
    }

    #[test]
    fn expression_root_lookup() {
        let globals: Object = serde_yaml::from_str(
            r#"
b: c
a:
  c: result
"#,
        )
        .unwrap();
        let mut var = Variable::with_literal("a");
        var.extend([Expression::Variable(Variable::with_expression(Expression::with_literal("b")))]);

        let runtime = RuntimeBuilder::new().build();
        let runtime = StackFrame::new(&runtime, &globals);
        let actual = var.evaluate(&runtime).unwrap();
        let actual = runtime.get(&actual).unwrap();
        assert_eq!(actual, ValueViewCmp::new(&"result"));
    }
}
