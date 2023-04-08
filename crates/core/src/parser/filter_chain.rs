use std::fmt;
use std::io::Write;

use super::Filter;
use crate::error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use crate::model::{ValueCow, ValueView};
use crate::runtime::Expression;
use crate::runtime::Renderable;
use crate::runtime::Runtime;
use crate::{ErrorKind, Value};

/// A `Value` expression.
#[derive(Debug)]
pub struct FilterChain {
    entry: Expression,
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    /// Create a new expression.
    pub fn new(entry: Expression, filters: Vec<Box<dyn Filter>>) -> Self {
        Self { entry, filters }
    }

    /// Process `Value` expression within `runtime`'s stack.
    pub fn evaluate<'s>(&'s self, runtime: &'s dyn Runtime) -> Result<ValueCow<'s>> {
        // take either the provided value or the value from the provided variable
        let mut entry = match self.entry.evaluate(runtime) {
            Ok(v) => v,
            Err(err) if self.filters.is_empty() => return Err(err),
            Err(err) => match err.kind() {
                // a missing value is not an error if there are filters
                // that come next to process it. They will decide if this
                // is appropriate or not (eg: the default filter)
                ErrorKind::UnknownIndex | ErrorKind::UnknownVariable => ValueCow::Owned(Value::Nil),
                _ => return Err(err),
            },
        };

        // apply all specified filters
        for filter in &self.filters {
            entry = filter
                .evaluate(entry.as_view(), runtime)
                .trace("Filter error")
                .context_key("filter")
                .value_with(|| format!("{}", filter).into())
                .context_key("input")
                .value_with(|| format!("{}", entry.source()).into())
                .map(ValueCow::Owned)?;
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
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let entry = self.evaluate(runtime)?;
        write!(writer, "{}", entry.render()).replace("Failed to render")?;
        Ok(())
    }
}
