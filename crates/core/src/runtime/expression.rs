use std::fmt;

use crate::error::{Error, Result};
use crate::model::Scalar;
use crate::model::Value;
use crate::model::ValueCow;
use crate::model::ValueView;
use std::sync::Arc;

use super::variable::Variable;
use super::AssignedRangeValue;
use super::Runtime;

/// An un-evaluated `Value`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Un-evaluated.
    Variable(Variable),
    /// Evaluated.
    Literal(Value),
    /// Range expression.
    Range(Box<(Expression, Expression)>),
}

impl Expression {
    /// Create an expression from a scalar literal.
    pub fn with_literal<S: Into<Scalar>>(literal: S) -> Self {
        Expression::Literal(Value::scalar(literal))
    }

    /// Convert into a literal if possible.
    pub fn into_literal(self) -> Option<Value> {
        match self {
            Expression::Literal(x) => Some(x),
            Expression::Variable(_) | Expression::Range(_) => None,
        }
    }

    /// Convert into a variable, if possible.
    pub fn into_variable(self) -> Option<Variable> {
        match self {
            Expression::Literal(_) | Expression::Range(_) => None,
            Expression::Variable(x) => Some(x),
        }
    }

    /// Convert to a `Value`.
    pub fn try_evaluate<'c>(&'c self, runtime: &'c dyn Runtime) -> Option<ValueCow<'c>> {
        match self {
            Expression::Literal(ref x) => Some(ValueCow::Borrowed(x)),
            Expression::Variable(ref x) => {
                let path = x.try_evaluate(runtime)?;
                runtime.try_get(&path)
            }
            Expression::Range(bounds) => evaluate_range(bounds, runtime).ok(),
        }
    }

    /// Convert to a `Value`.
    pub fn evaluate<'c>(&'c self, runtime: &'c dyn Runtime) -> Result<ValueCow<'c>> {
        let val = match self {
            Expression::Literal(ref x) => ValueCow::Borrowed(x),
            Expression::Variable(ref x) => {
                let path = x.evaluate(runtime)?;
                runtime.get(&path)?
            }
            Expression::Range(bounds) => evaluate_range(bounds, runtime)?,
        };
        Ok(val)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(ref x) => write!(f, "{}", x.source()),
            Expression::Variable(ref x) => write!(f, "{}", x),
            Expression::Range(bounds) => write!(f, "({}..{})", bounds.0, bounds.1),
        }
    }
}

fn evaluate_range<'c>(
    bounds: &'c (Expression, Expression),
    runtime: &'c dyn Runtime,
) -> Result<ValueCow<'c>> {
    let start = range_bound_to_i64(bounds.0.evaluate(runtime)?.as_view())?;
    let stop = range_bound_to_i64(bounds.1.evaluate(runtime)?.as_view())?;
    Ok(ValueCow::Shared(Arc::new(AssignedRangeValue::new(
        start, stop,
    ))))
}

fn range_bound_to_i64(value: &dyn ValueView) -> Result<i64> {
    if value.is_nil() {
        return Ok(0);
    }

    let Some(scalar) = value.as_scalar() else {
        return Err(invalid_integer_error());
    };

    if let Some(integer) = scalar.to_integer() {
        return Ok(integer);
    }

    if scalar.is_string() {
        let string = scalar.to_kstr();
        return Ok(string
            .as_str()
            .parse::<f64>()
            .map(|float| float as i64)
            .unwrap_or(0));
    }

    Err(invalid_integer_error())
}

fn invalid_integer_error() -> Error {
    Error::with_msg("invalid integer")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Object;
    use crate::runtime::RuntimeBuilder;

    #[test]
    fn range_bound_accepts_nil_and_strings_like_ruby_liquid() {
        assert_eq!(range_bound_to_i64(&Value::Nil).unwrap(), 0);
        assert_eq!(
            range_bound_to_i64(&Value::scalar("invalid integer")).unwrap(),
            0
        );
        assert_eq!(range_bound_to_i64(&Value::scalar("3.14")).unwrap(), 3);
        assert_eq!(range_bound_to_i64(&Value::scalar("4")).unwrap(), 4);
    }

    #[test]
    fn range_bound_rejects_non_string_non_integer_scalars() {
        let err = range_bound_to_i64(&Value::scalar(1.5f64)).unwrap_err();
        assert_eq!(err.to_string(), "liquid: invalid integer\n");
    }

    #[test]
    fn range_bound_rejects_non_scalars() {
        let err = range_bound_to_i64(&Value::Object(Object::new())).unwrap_err();
        assert_eq!(err.to_string(), "liquid: invalid integer\n");
    }

    #[test]
    fn expression_range_propagates_invalid_integer_errors() {
        let runtime = RuntimeBuilder::new().build();
        let expression = Expression::Range(Box::new((
            Expression::Literal(Value::scalar(1.5f64)),
            Expression::with_literal(3i64),
        )));

        let err = expression.evaluate(&runtime).unwrap_err();
        assert_eq!(err.to_string(), "liquid: invalid integer\n");
    }

    #[test]
    fn expression_range_truncates_numeric_strings() {
        let runtime = RuntimeBuilder::new().build();
        let expression = Expression::Range(Box::new((
            Expression::Literal(Value::scalar("3.14")),
            Expression::with_literal(5i64),
        )));

        let value = expression.evaluate(&runtime).unwrap();
        assert_eq!(value.to_kstr().as_str(), "3..5");
    }
}
