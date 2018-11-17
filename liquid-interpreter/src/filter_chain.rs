use std::fmt;
use std::io::Write;

use itertools;

use error::{Error, Result, ResultLiquidChainExt, ResultLiquidExt};
use value::Value;

use super::Context;
use super::Expression;
use super::Renderable;

/// A `Value` filter.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterCall {
    name: String,
    arguments: Vec<Expression>,
}

impl FilterCall {
    /// Create filter expression.
    pub fn new(name: &str, arguments: Vec<Expression>) -> FilterCall {
        FilterCall {
            name: name.to_owned(),
            arguments,
        }
    }
}

impl fmt::Display for FilterCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name,
            itertools::join(&self.arguments, ", ")
        )
    }
}

/// A `Value` expression.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterChain {
    entry: Expression,
    filters: Vec<FilterCall>,
}

impl FilterChain {
    /// Create a new expression.
    pub fn new(entry: Expression, filters: Vec<FilterCall>) -> Self {
        Self { entry, filters }
    }

    /// Process `Value` expression within `context`'s stack.
    pub fn evaluate(&self, context: &Context) -> Result<Value> {
        // take either the provided value or the value from the provided variable
        let mut entry = self.entry.evaluate(context)?;

        // apply all specified filters
        for filter in &self.filters {
            let f = context.get_filter(&filter.name).ok_or_else(|| {
                Error::with_msg("Unsupported filter").context("filter", filter.name.clone())
            })?;

            let arguments: Result<Vec<Value>> = filter
                .arguments
                .iter()
                .map(|a| a.evaluate(context))
                .collect();
            let arguments = arguments?;
            entry = f
                .filter(&entry, &*arguments)
                .chain("Filter error")
                .context_with(|| ("filter".to_owned(), format!("{}", self)))
                .context_with(|| ("input".to_owned(), format!("{}", &entry)))
                .context_with(|| ("args".to_owned(), itertools::join(&arguments, ", ")))?;
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
        write!(writer, "{}", entry).chain("Failed to render")?;
        Ok(())
    }
}
