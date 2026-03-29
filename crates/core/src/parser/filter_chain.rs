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

    fn apply_filters<'s>(
        &'s self,
        mut entry: ValueCow<'s>,
        runtime: &'s dyn Runtime,
    ) -> Result<ValueCow<'s>> {
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

    /// Process `Value` expression within `runtime`'s stack.
    pub fn evaluate<'s>(&'s self, runtime: &'s dyn Runtime) -> Result<ValueCow<'s>> {
        // take either the provided value or the value from the provided variable
        let entry = self.entry.evaluate(runtime)?;
        self.apply_filters(entry, runtime)
    }

    /// Process `Value` expression within `runtime`'s stack for existence-style checks.
    ///
    /// Missing entries are treated as `nil` so filters still run, matching Liquid's
    /// behavior for expressions like `{% if missing | upcase %}`.
    pub fn try_evaluate<'s>(&'s self, runtime: &'s dyn Runtime) -> Result<ValueCow<'s>> {
        let entry = self.entry.try_evaluate(runtime).unwrap_or_default();
        self.apply_filters(entry, runtime)
    }

    /// Process a comparison operand.
    ///
    /// Plain operands keep `evaluate`'s strict missing-variable behavior so
    /// `{% if missing == 1 %}` still reports a render error. Filtered operands
    /// use `try_evaluate` so a missing input becomes `nil` and the filter chain
    /// still runs, matching Liquid behavior for cases like
    /// `{% if missing | upcase == "" %}`.
    pub fn evaluate_comparison_operand<'s>(
        &'s self,
        runtime: &'s dyn Runtime,
    ) -> Result<ValueCow<'s>> {
        if self.filters.is_empty() {
            self.evaluate(runtime)
        } else {
            self.try_evaluate(runtime)
        }
    }
}

impl fmt::Display for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.entry)?;
        // The old join-based formatter produced a trailing " | " for zero-filter chains.
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
