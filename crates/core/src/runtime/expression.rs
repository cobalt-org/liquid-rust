use std::fmt;

use liquid_error::Result;
use liquid_value::Scalar;
use liquid_value::Value;
use liquid_value::ValueCow;
use liquid_value::ValueView;

use super::Runtime;
use super::variable::Variable;

/// An un-evaluated `Value`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Un-evaluated.
    Variable(Variable),
    /// Evaluated.
    Literal(Value),
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
            Expression::Variable(_) => None,
        }
    }

    /// Convert into a variable, if possible.
    pub fn into_variable(self) -> Option<Variable> {
        match self {
            Expression::Literal(_) => None,
            Expression::Variable(x) => Some(x),
        }
    }

    /// Convert to a `Value`.
    pub fn try_evaluate<'c>(&'c self, runtime: &'c Runtime<'_>) -> Option<ValueCow<'c>> {
        match self {
            Expression::Literal(ref x) => Some(ValueCow::Borrowed(x)),
            Expression::Variable(ref x) => {
                let path = x.try_evaluate(runtime)?;
                runtime.stack().try_get(&path)
            }
        }
    }

    /// Convert to a `Value`.
    pub fn evaluate<'c>(&'c self, runtime: &'c Runtime<'_>) -> Result<ValueCow<'c>> {
        let val = match self {
            Expression::Literal(ref x) => ValueCow::Borrowed(x),
            Expression::Variable(ref x) => {
                let path = x.evaluate(runtime)?;
                runtime.stack().get(&path)?
            }
        };
        Ok(val)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(ref x) => write!(f, "{}", x.source()),
            Expression::Variable(ref x) => write!(f, "{}", x),
        }
    }
}
