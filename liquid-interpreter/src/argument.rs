use std::fmt;

use error::Result;
use value::Value;

use super::Context;
use variable::Variable;

/// An un-evaluated `Value`.
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// Un-evaluated.
    Var(Variable),
    /// Evaluated.
    Val(Value),
}

impl Argument {
    /// Convert to a `Value`.
    pub fn evaluate(&self, context: &Context) -> Result<Value> {
        let val = match *self {
            Argument::Val(ref x) => x.clone(),
            Argument::Var(ref x) => context
                .stack()
                .get(x.path())?
                .clone(),
        };
        Ok(val)
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Argument::Val(ref x) => write!(f, "{}", x),
            Argument::Var(ref x) => write!(f, "{}", x),
        }
    }
}
