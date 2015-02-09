use Renderable;
use Context;
use value::Value;
use variable::Variable;

pub struct FilterPrototype{
    name: String,
    arguments: Vec<Value>
}

pub enum VarOrVal {
    Var(Variable),
    Val(Value)
}

impl FilterPrototype {
    pub fn new(name: &str, arguments: Vec<Value>) -> FilterPrototype {
        FilterPrototype{name: name.to_string(), arguments: arguments}
    }
}

pub struct Output{
    entry: VarOrVal,
    filters: Vec<FilterPrototype>
}

impl Renderable for Output {
    fn render (&self, context: &mut Context) -> Option<String>{
        let mut entry = match self.entry  {
            VarOrVal::Val(ref x) => x.render(context).unwrap(),
            VarOrVal::Var(ref x) => x.render(context).unwrap()
        };
        for filter in self.filters.iter(){
            let f = match context.filters.get(&filter.name) {
                Some(x) => x,
                None => panic!("Filter not implemented")
            };
            entry = f(&entry[]);
        }
        Some(entry)
    }
}

impl Output {
    pub fn new(entry: VarOrVal, filters: Vec<FilterPrototype>) -> Output {
        Output{entry: entry, filters: filters}
    }
}

