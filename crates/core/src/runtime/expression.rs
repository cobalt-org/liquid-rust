use std::fmt;

use crate::error::Result;
use crate::model::Object;
use crate::model::Scalar;
use crate::model::Value;
use crate::model::ValueCow;
use crate::model::ValueView;

use super::variable::Variable;
use super::Runtime;

use std::collections::HashMap;

/// An un-evaluated `Value`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Un-evaluated.
    Variable(Variable),
    /// Evaluated.
    Literal(Value),
    /// Used for evaluating object literals,
    ObjectLiteral(ObjectLiteral),
}

type ObjectLiteral = HashMap<String, Expression>;

impl Expression {
    /// Create an expression from a scalar literal.
    pub fn with_literal<S: Into<Scalar>>(literal: S) -> Self {
        Expression::Literal(Value::scalar(literal))
    }

    /// Creates an expression from an object literal (used when parsing filter
    /// arguments)
    pub fn with_object_literal(object_literal_expr: ObjectLiteral) -> Self {
        Expression::ObjectLiteral(object_literal_expr)
    }

    /// Convert into a literal if possible.
    pub fn into_literal(self) -> Option<Value> {
        match self {
            Expression::Literal(x) => Some(x),
            Expression::Variable(_) => None,
            Expression::ObjectLiteral(_) => None,
        }
    }

    /// Convert into a variable, if possible.
    pub fn into_variable(self) -> Option<Variable> {
        match self {
            Expression::Literal(_) => None,
            Expression::Variable(x) => Some(x),
            Expression::ObjectLiteral(_) => None,
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
            Expression::ObjectLiteral(ref obj_lit) => {
                let obj = obj_lit
                    .iter()
                    .map(|(key, expr)| match expr.try_evaluate(runtime) {
                        Some(result) => (key.into(), result.to_value()),
                        None => (key.into(), Value::Nil),
                    })
                    .collect::<Object>();
                Some(ValueCow::Owned(obj.to_value()))
            }
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
            Expression::ObjectLiteral(obj_lit) => obj_lit
                .iter()
                .map(|(key, expr)| (key.into(), expr.evaluate(runtime).unwrap().to_value()))
                .collect::<Object>()
                .into(),
        };
        Ok(val)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(ref x) => write!(f, "{}", x.source()),
            Expression::Variable(ref x) => write!(f, "{}", x),
            Expression::ObjectLiteral(ref x) => write!(f, "{:?}", x),
        }
    }
}
