use Renderable;
use context::Context;
use value::Value;
use variable::Variable;
use error::{Error, Result};

#[derive(Debug)]
pub struct FilterPrototype {
    name: String,
    arguments: Vec<Value>,
}

#[derive(Debug)]
pub enum VarOrVal {
    Var(Variable),
    Val(Value),
}

impl FilterPrototype {
    pub fn new(name: &str, arguments: Vec<Value>) -> FilterPrototype {
        FilterPrototype {
            name: name.to_owned(),
            arguments: arguments,
        }
    }
}

pub struct Output {
    entry: VarOrVal,
    filters: Vec<FilterPrototype>,
}

impl Renderable for Output {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        // take either the provided value or the value from the provided variable
        let mut entry = match self.entry {
            VarOrVal::Val(ref x) => x.clone(),
            VarOrVal::Var(ref x) => {
                context.get_val(&*x.name()).cloned().unwrap_or(Value::Str("".to_owned()))
            }
        };

        // apply all specified filters
        for filter in &self.filters {
            let f = try!(context.get_filter(&filter.name)
                                .ok_or(Error::Render(format!("Filter {} not implemented",
                                                             &filter.name))));
            entry = try!(f(&entry, &filter.arguments));
        }

        entry.render(context)
    }
}

impl Output {
    pub fn new(entry: VarOrVal, filters: Vec<FilterPrototype>) -> Output {
        Output {
            entry: entry,
            filters: filters,
        }
    }
}
