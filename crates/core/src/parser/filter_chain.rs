use std::sync;
use std::fmt;
use std::io::Write;

use super::ParsedFilter;
use super::ParseFilter;
use super::PluginRegistry;
use crate::error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use crate::model::{ValueCow, ValueView};
use crate::runtime::{Expression, Variable};
use crate::runtime::Renderable;
use crate::runtime::Runtime;

/// A `Value` expression.
pub struct FilterChain {
    entry: Expression,
    filters: Vec<ParsedFilter>,
    fallback_filters: sync::Arc<PluginRegistry<Box<dyn ParseFilter>>>,
}

impl FilterChain {
    /// Create a new expression.
    pub fn new(
        entry: Expression,
        filters: Vec<ParsedFilter>,
        fallback_filters: sync::Arc<PluginRegistry<Box<dyn ParseFilter>>>,
    ) -> Self {
        Self {
            entry,
            filters,
            fallback_filters,
        }
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

    /// Process the expression while tracking whether each filter preserved the
    /// source value identity for assign-range semantics.
    pub fn evaluate_with_identity<'s>(
        &'s self,
        runtime: &'s dyn Runtime,
    ) -> Result<(ValueCow<'s>, bool)> {
        let entry = self.entry.evaluate(runtime)?;
        self.apply_filters_with_identity(entry, runtime)
    }

    /// Returns the underlying variable when the chain is an unfiltered variable lookup.
    pub fn as_plain_variable(&self) -> Option<&Variable> {
        if self.filters.is_empty() {
            self.source_variable()
        } else {
            None
        }
    }

    /// Returns the source variable for the chain entry, even when filters are present.
    pub fn source_variable(&self) -> Option<&Variable> {
        match &self.entry {
            Expression::Variable(variable) => Some(variable),
            Expression::Literal(_) => None,
        }
    }

    /// Apply each parsed filter in order, preserving the current value so filter
    /// failures can report both the filter name and the input that triggered it.
    fn apply_filters<'s>(
        &'s self,
        entry: ValueCow<'s>,
        runtime: &'s dyn Runtime,
    ) -> Result<ValueCow<'s>> {
        self.apply_filters_with_identity(entry, runtime).map(|(entry, _)| entry)
    }

    fn apply_filters_with_identity<'s>(
        &'s self,
        mut entry: ValueCow<'s>,
        runtime: &'s dyn Runtime,
    ) -> Result<(ValueCow<'s>, bool)> {
        let mut preserved_identity = true;
        for filter in &self.filters {
            let preserves_input = match filter {
                ParsedFilter::Compiled(filter) => {
                    filter.preserves_input_identity(entry.as_view(), runtime)?
                }
                ParsedFilter::Deferred(_) => false,
            };
            let evaluated = match filter {
                ParsedFilter::Compiled(filter) => filter.evaluate(entry.as_view(), runtime),
                ParsedFilter::Deferred(filter) => {
                    runtime.evaluate_filter(filter, entry.as_view(), self.fallback_filters.as_ref())
                }
            };

            entry = ValueCow::Owned(
                evaluated
                    .trace("Filter error")
                    .context_key("filter")
                    .value_with(|| format!("{}", filter).into())
                    .context_key("input")
                    .value_with(|| format!("{}", entry.source()).into())?,
            );
            preserved_identity &= preserves_input;
        }

        Ok((entry, preserved_identity))
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

impl fmt::Debug for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterChain")
            .field("entry", &self.entry)
            .field("filters", &self.filters)
            .finish()
    }
}

impl Renderable for FilterChain {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let entry = self.evaluate(runtime)?;
        write!(writer, "{}", entry.render()).replace("Failed to render")?;
        Ok(())
    }
}
