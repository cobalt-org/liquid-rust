use error::{Error, Result};
use value::Value;

use super::Context;
use super::Renderable;
use super::variable::Variable;

#[derive(Debug, PartialEq)]
pub struct FilterPrototype {
    name: String,
    arguments: Vec<Argument>,
}

#[derive(Debug, PartialEq)]
pub enum Argument {
    Var(Variable),
    Val(Value),
}

impl FilterPrototype {
    pub fn new(name: &str, arguments: Vec<Argument>) -> FilterPrototype {
        FilterPrototype {
            name: name.to_owned(),
            arguments: arguments,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Output {
    entry: Argument,
    filters: Vec<FilterPrototype>,
}

impl Renderable for Output {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let entry = self.apply_filters(context)?;
        Ok(Some(entry.to_string()))
    }
}

impl Output {
    pub fn new(entry: Argument, filters: Vec<FilterPrototype>) -> Output {
        Output {
            entry: entry,
            filters: filters,
        }
    }

    pub fn apply_filters(&self, context: &Context) -> Result<Value> {
        // take either the provided value or the value from the provided variable
        let mut entry = match self.entry {
            Argument::Val(ref x) => x.clone(),
            Argument::Var(ref x) => context.get_val_by_index(x.indexes().iter())?.clone(),
        };

        // apply all specified filters
        for filter in &self.filters {
            let f =
                context
                    .get_filter(&filter.name)
                    .ok_or_else(|| {
                                    Error::Render(format!("Filter {} not implemented",
                                                          &filter.name))
                                })?;

            let mut arguments = Vec::new();
            for arg in &filter.arguments {
                match *arg {
                    Argument::Var(ref x) => {
                        let val = context.get_val_by_index(x.indexes().iter())?.clone();
                        arguments.push(val);
                    }
                    Argument::Val(ref x) => arguments.push(x.clone()),
                }
            }
            entry = f.filter(&entry, &*arguments)?;
        }

        Ok(entry)
    }
}
