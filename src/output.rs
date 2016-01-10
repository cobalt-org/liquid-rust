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
            name: name.to_string(),
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
        let mut entry = match self.entry {
            VarOrVal::Val(ref x) => try!(x.render(context)).unwrap_or("".to_owned()),
            VarOrVal::Var(ref x) => try!(x.render(context)).unwrap_or("".to_owned()),
        };
        for filter in self.filters.iter() {
            let f = match context.filters.get(&filter.name) {
                Some(x) => x,
                None => return Err(Error::Render(format!("Filter {} not implemented", &filter.name))),
            };
            entry = f(&entry, &filter.arguments);
        }
        Ok(Some(entry))
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
