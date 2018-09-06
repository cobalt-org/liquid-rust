use std::fmt;
use std::io::Write;

use itertools;

use error::{Error, Result, ResultLiquidChainExt, ResultLiquidExt};
use value::Value;

use super::Argument;
use super::Context;
use super::Renderable;

#[derive(Clone, Debug, PartialEq)]
pub struct FilterPrototype {
    name: String,
    arguments: Vec<Argument>,
}

impl FilterPrototype {
    pub fn new(name: &str, arguments: Vec<Argument>) -> FilterPrototype {
        FilterPrototype {
            name: name.to_owned(),
            arguments,
        }
    }
}

impl fmt::Display for FilterPrototype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name,
            itertools::join(&self.arguments, ", ")
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    entry: Argument,
    filters: Vec<FilterPrototype>,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            self.entry,
            itertools::join(&self.filters, " | ")
        )
    }
}

impl Renderable for Output {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let entry = self.apply_filters(context)?;
        write!(writer, "{}", entry).chain("Failed to render")?;
        Ok(())
    }
}

impl Output {
    pub fn new(entry: Argument, filters: Vec<FilterPrototype>) -> Output {
        Output { entry, filters }
    }

    pub fn apply_filters(&self, context: &Context) -> Result<Value> {
        // take either the provided value or the value from the provided variable
        let mut entry = self.entry.evaluate(context)?;

        // apply all specified filters
        for filter in &self.filters {
            let f = context.get_filter(&filter.name).ok_or_else(|| {
                Error::with_msg("Unsupported filter").context("filter", &filter.name)
            })?;

            let arguments: Result<Vec<Value>> = filter
                .arguments
                .iter()
                .map(|a| a.evaluate(context))
                .collect();
            let arguments = arguments?;
            entry = f.filter(&entry, &*arguments)
                .chain("Filter error")
                .context("input", &entry)
                .context_with(|| ("args".into(), itertools::join(&arguments, ", ")))?;
        }

        Ok(entry)
    }
}
