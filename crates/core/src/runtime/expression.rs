use std::fmt;

use crate::error::Result;
use crate::model::Scalar;
use crate::model::Value;
use crate::model::ValueCow;
use crate::model::ValueView;

use super::variable::Variable;
use super::Runtime;

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
    pub fn try_evaluate<'c>(&'c self, runtime: &'c dyn Runtime) -> Option<ValueCow<'c>> {
        match self {
            Expression::Literal(ref x) => Some(ValueCow::Borrowed(x)),
            Expression::Variable(ref x) => {
                let path = x.try_evaluate(runtime)?;
                runtime.try_get(&path)
            }
        }
    }

    /// Convert to a `Value`.
    pub fn evaluate<'c>(&'c self, runtime: &'c dyn Runtime) -> Result<ValueCow<'c>> {
        let val = match self {
            Expression::Literal(ref x) => ValueCow::Borrowed(x),
            Expression::Variable(ref x) => {
                let path = x.evaluate(runtime)?;

                match runtime.render_mode() {
                    super::RenderingMode::Lax => {
                        runtime.try_get(&path).unwrap_or_else(|| Value::Nil.into())
                    }
                    _ => runtime.get(&path)?,
                }
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

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::Object;
    use crate::model::Value;
    use crate::runtime::RenderingMode;
    use crate::runtime::RuntimeBuilder;
    use crate::runtime::StackFrame;

    #[test]
    fn test_rendering_mode() {
        let globals = Object::new();
        let expression = Expression::Variable(Variable::with_literal("test"));

        let runtime = RuntimeBuilder::new()
            .set_render_mode(RenderingMode::Strict)
            .build();
        let runtime = StackFrame::new(&runtime, &globals);
        assert_eq!(expression.evaluate(&runtime).is_err(), true);

        let runtime = RuntimeBuilder::new()
            .set_render_mode(RenderingMode::Lax)
            .build();
        let runtime = StackFrame::new(&runtime, &globals);
        assert_eq!(expression.evaluate(&runtime).unwrap(), Value::Nil);
    }
}
