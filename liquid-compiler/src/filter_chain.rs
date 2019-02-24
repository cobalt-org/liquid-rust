use std::fmt;
use std::io::Write;

use itertools;

use super::Filter;
use liquid_error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_interpreter::Renderable;
use liquid_value::Value;

/// A `Value` expression.
#[derive(Debug)]
pub struct FilterChain {
    entry: Expression,
    filters: Vec<Box<Filter>>,
}

impl FilterChain {
    /// Create a new expression.
    pub fn new(entry: Expression, filters: Vec<Box<Filter>>) -> Self {
        Self { entry, filters }
    }

    /// Process `Value` expression within `context`'s stack.
    pub fn evaluate(&self, context: &Context) -> Result<Value> {
        // take either the provided value or the value from the provided variable
        let mut entry = self.entry.evaluate(context)?.to_owned();

        // apply all specified filters
        for filter in &self.filters {
            entry = filter
                .evaluate(&entry, context)
                .trace("Filter error")
                .context_key("filter")
                .value_with(|| format!("{}", filter).into())
                .context_key("input")
                .value_with(|| format!("{}", entry.source()).into())?;
        }

        Ok(entry)
    }
}

impl fmt::Display for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            self.entry,
            itertools::join(&self.filters, " | ")
        )
    }
}

impl Renderable for FilterChain {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let entry = self.evaluate(context)?;
        write!(writer, "{}", entry.to_str()).replace("Failed to render")?;
        Ok(())
    }
}
