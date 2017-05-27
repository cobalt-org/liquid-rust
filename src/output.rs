use Renderable;
use context::Context;
use value::Value;
use variable::Variable;
use error::{Error, Result};

#[derive(Debug, PartialEq)]
pub struct FilterPrototype {
    name: String,
    arguments: Vec<Argument>,
    named_arguments: Vec<(String, Argument)>,
}

#[derive(Debug, PartialEq)]
pub enum Argument {
    Var(Variable),
    Val(Value),
}

impl FilterPrototype {
    pub fn new(name: &str,
               arguments: Vec<Argument>,
               named_arguments: Vec<(String, Argument)>)
               -> FilterPrototype {
        FilterPrototype {
            name: name.to_owned(),
            arguments: arguments,
            named_arguments: named_arguments,
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
        let entry = try!(self.apply_filters(context));
        entry.render(context)
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
            Argument::Var(ref x) => {
                context
                    .get_val(&*x.name())
                    .cloned()
                    .unwrap_or_else(|| Value::Str("".to_owned()))
            }
        };

        // apply all specified filters
        for filter in &self.filters {
            let f = try!(context
                             .get_filter(&filter.name)
                             .ok_or_else(|| {
                                             Error::Render(format!("Filter {} not implemented",
                                                                   &filter.name))
                                         }));

            let mut arguments = Vec::new();
            for arg in &filter.arguments {
                match *arg {
                    Argument::Var(ref x) => {
                        let val = try!(context.get_val(&*x.name())
                            .cloned()
                            .ok_or_else(|| {
                                Error::Render(format!("undefined variable {}", x.name()))
                            }));
                        arguments.push(val);
                    }
                    Argument::Val(ref x) => arguments.push(x.clone()),
                }
            }
            entry = try!(f(&entry, &*arguments));
        }

        Ok(entry)
    }
}
