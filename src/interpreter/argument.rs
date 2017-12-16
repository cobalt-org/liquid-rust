use std::fmt;

use error::Result;
use value::Value;

use super::Context;
use super::variable::Variable;

#[derive(Debug, PartialEq)]
pub enum Argument {
    Var(Variable),
    Val(Value),
}

impl Argument {
    pub fn evaluate(&self, context: &Context) -> Result<Value> {
        let val = match self {
            &Argument::Val(ref x) => x.clone(),
            &Argument::Var(ref x) => context.get_val_by_index(x.indexes().iter())?.clone(),
        };
        Ok(val)
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Argument::Val(ref x) => write!(f, "{}", x),
            &Argument::Var(ref x) => write!(f, "{}", x),
        }
    }
}
