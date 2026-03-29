use std::fmt;
use std::io::Write;

use super::Filter;
use crate::error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use crate::model::{ValueCow, ValueView};
use crate::runtime::Expression;
use crate::runtime::Renderable;
use crate::runtime::Runtime;

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
        let mut entry = self.entry.evaluate(runtime)?;

        // apply all specified filters
        for filter in &self.filters {
            entry = ValueCow::Owned(
                filter
                    .evaluate(entry.as_view(), runtime)
                    .trace("Filter error")
                    .context_key("filter")
                    .value_with(|| format!("{}", filter).into())
                    .context_key("input")
                    .value_with(|| format!("{}", entry.source()).into())?,
            );
        }

        Ok(entry)
    }

    /// Process `Value` expression within `runtime`'s stack, returning `None` when the
    /// chain entry does not resolve and propagating filter failures otherwise.
    pub fn try_evaluate<'s>(&'s self, runtime: &'s dyn Runtime) -> Result<Option<ValueCow<'s>>> {
        let Some(mut entry) = self.entry.try_evaluate(runtime) else {
            return Ok(None);
        };

        for filter in &self.filters {
            entry = ValueCow::Owned(
                filter
                    .evaluate(entry.as_view(), runtime)
                    .trace("Filter error")
                    .context_key("filter")
                    .value_with(|| format!("{}", filter).into())
                    .context_key("input")
                    .value_with(|| format!("{}", entry.source()).into())?,
            );
        }

        Ok(Some(entry))
    }
}

impl fmt::Display for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.entry)?;
        self.filters
            .iter()
            .try_for_each(|filter| write!(f, " | {}", filter))
    }
}

impl Renderable for FilterChain {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let entry = self.evaluate(runtime)?;
        write!(writer, "{}", entry.render()).replace("Failed to render")?;
        Ok(())
    }
}
