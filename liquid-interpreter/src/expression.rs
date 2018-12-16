use std::fmt;

use liquid_error::Result;
use liquid_value::Scalar;
use liquid_value::Value;

use super::Context;
use variable::Variable;

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
    pub fn try_evaluate<'c>(&'c self, context: &'c Context) -> Option<&'c Value> {
        let val = match *self {
            Expression::Literal(ref x) => &x,
            Expression::Variable(ref x) => {
                let path = x.try_evaluate(context)?;
                context.stack().try_get(&path)?
            }
        };
        Some(val)
    }

    /// Convert to a `Value`.
    pub fn evaluate<'c>(&'c self, context: &'c Context) -> Result<&'c Value> {
        let val = match *self {
            Expression::Literal(ref x) => x,
            Expression::Variable(ref x) => {
                let path = x.evaluate(context)?;
                context.stack().get(&path)?
            }
        };
        Ok(val)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::Literal(ref x) => write!(f, "{}", x.source()),
            Expression::Variable(ref x) => write!(f, "{}", x),
        }
    }
}
