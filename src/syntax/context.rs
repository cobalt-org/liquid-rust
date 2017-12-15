use std::collections::HashMap;

use error::{Result, Error};
use super::Filter;
use super::Token;
use super::{Value, Object};


#[derive(Clone)]
pub enum Interrupt {
    Continue,
    Break,
}

#[derive(Default)]
pub struct Context {
    stack: Vec<Object>,
    globals: Object,

    /// The current interrupt state. The interrupt state is used by
    /// the `break` and `continue` tags to halt template rendering
    /// at a given point and unwind the `render` call stack until
    /// it reaches an enclosing `for_loop`. At that point the interrupt
    /// is cleared, and the `for_loop` carries on processing as directed.
    interrupt: Option<Interrupt>,

    /// The indices of all the cycles encountered during rendering.
    cycles: HashMap<String, usize>,

    // Public for backwards compatability
    filters: HashMap<String, Box<Filter>>,
}

impl Context {
    /// Creates a new, empty rendering context.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_values(mut self, values: Object) -> Self {
        self.globals = values;
        self
    }

    pub fn with_filters(mut self, filters: HashMap<String, Box<Filter>>) -> Self {
        self.filters = filters;
        self
    }

    pub fn cycle_element(&mut self, name: &str, values: &[Token]) -> Result<Option<Value>> {
        let index = {
            let i = self.cycles.entry(name.to_owned()).or_insert(0);
            let j = *i;
            *i = (*i + 1) % values.len();
            j
        };

        if index >= values.len() {
            return Err(Error::Render(format!("cycle index {} out of bounds {}",
                                             index,
                                             values.len())));
        }

        self.evaluate(&values[index])
    }

    /// Only add the given filter to the context if a filter with this name doesn't already exist.
    pub fn maybe_add_filter(&mut self, name: &str, filter: Box<Filter>) {
        if !self.filters.contains_key(name) {
            self.add_filter(name, filter)
        }
    }

    pub fn add_filter(&mut self, name: &str, filter: Box<Filter>) {
        self.filters.insert(name.to_owned(), filter);
    }

    pub fn get_filter<'b>(&'b self, name: &str) -> Option<&'b Filter> {
        self.filters.get(name).map(|f| f.as_ref())
    }

    pub fn interrupted(&self) -> bool {
        self.interrupt.is_some()
    }

    /// Sets the interrupt state. Any previous state is obliterated.
    pub fn set_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt = Some(interrupt);
    }

    /// Fetches and clears the interrupt state.
    pub fn pop_interrupt(&mut self) -> Option<Interrupt> {
        let rval = self.interrupt.clone();
        self.interrupt = None;
        rval
    }

    /// Creates a new variable scope chained to a parent scope.
    fn push_scope(&mut self) {
        self.stack.push(Object::new());
    }

    /// Removes the topmost stack frame from the local variable stack.
    ///
    /// # Panics
    ///
    /// This method will panic if popping the topmost frame results in an
    /// empty stack. Given that a context is created with a top-level stack
    /// frame already in place, empyting the stack should never happen in a
    /// well-formed program.
    fn pop_scope(&mut self) {
        if self.stack.pop().is_none() {
            panic!("Pop leaves empty stack")
        };
    }

    /// Sets up a new stack frame, executes the supplied function and then
    /// tears the stack frame down before returning the function's result
    /// to the caller.
    pub fn run_in_scope<RvalT, FnT>(&mut self, f: FnT) -> RvalT
        where FnT: FnOnce(&mut Context) -> RvalT
    {
        self.push_scope();
        let result = f(self);
        self.pop_scope();
        result
    }

    /// Internal part of get_val. Walks the scope stack to try and find the
    /// reqested variable, and failing that checks the global pool.
    fn get<'a>(&'a self, name: &str) -> Option<&'a Value> {
        for frame in self.stack.iter().rev() {
            if let rval @ Some(_) = frame.get(name) {
                return rval;
            }
        }
        self.globals.get(name)
    }

    /// Gets a value from the rendering context. The name value can be a
    /// dot-separated path to a value. A value will only be returned if
    /// each link in the chain (excluding the final name) refers to a
    /// value of type Object.
    pub fn get_val<'b>(&'b self, name: &str) -> Option<&'b Value> {
        let mut path = name.split('.');
        let key = path.next().unwrap_or("");
        let mut rval = self.get(key);

        // walk the chain of Object values, as specified by the path
        // passed in name
        for id in path {
            match rval {
                Some(&Value::Object(ref x)) => rval = x.get(id),
                _ => return None,
            }
        }

        rval
    }

    /// Sets a value in the global context.
    pub fn set_val(&mut self, name: &str, val: Value) -> Option<Value> {
        self.globals.insert(name.to_owned(), val)
    }

    /// Translates a Token to a Value, looking it up in the context if
    /// necessary
    pub fn evaluate(&self, t: &Token) -> Result<Option<Value>> {
        match *t {
            Token::NumberLiteral(f) => Ok(Some(Value::Num(f))),
            Token::StringLiteral(ref s) => Ok(Some(Value::Str(s.clone()))),
            Token::BooleanLiteral(b) => Ok(Some(Value::Bool(b))),
            Token::Identifier(ref id) => Ok(self.get_val(id).cloned()),
            _ => {
                let msg = format!("Cannot evaluate {}", t);
                Err(Error::Other(msg))
            }
        }
    }

    /// Sets a value to the rendering context.
    /// Note that it needs to be wrapped in a liquid::Value.
    ///
    /// # Panics
    ///
    /// Panics if there is no frame on the local values stack. Context
    /// instances are created with a top-level stack frame in place, so
    /// this should never happen in a well-formed program.
    pub fn set_local_val(&mut self, name: &str, val: Value) -> Option<Value> {
        match self.stack.last_mut() {
            Some(frame) => frame.insert(name.to_owned(), val),
            None => panic!("Cannot insert into an empty stack"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_val() {
        let mut ctx = Context::new();
        let mut post = HashMap::new();
        post.insert("number".to_owned(), Value::Num(42f32));
        ctx.set_val("post", Value::Object(post));
        assert_eq!(ctx.get_val("post.number").unwrap(), &Value::Num(42f32));
    }

    #[test]
    fn scoped_variables() {
        let mut ctx = Context::new();
        ctx.set_val("test", Value::Num(42f32));
        assert_eq!(ctx.get_val("test").unwrap(), &Value::Num(42f32));

        ctx.run_in_scope(|new_scope| {
            // assert that values are chained to the parent scope
            assert_eq!(new_scope.get_val("test").unwrap(), &Value::Num(42f32));

            // set a new local value, and assert that it overrides the previous value
            new_scope.set_local_val("test", Value::Num(3.14f32));
            assert_eq!(new_scope.get_val("test").unwrap(), &Value::Num(3.14f32));

            // sat a new val that we will pick up outside the scope
            new_scope.set_val("global", Value::str("some value"));
        });

        // assert that the value has reverted to the old one
        assert_eq!(ctx.get_val("test").unwrap(), &Value::Num(42f32));
        assert_eq!(ctx.get_val("global").unwrap(), &Value::str("some value"));
    }

    #[test]
    fn evaluate_handles_string_literals() {
        let ctx = Context::new();
        let t = Token::StringLiteral("hello".to_owned());
        assert_eq!(ctx.evaluate(&t).unwrap(), Some(Value::str("hello")));
    }

    #[test]
    fn evaluate_handles_number_literals() {
        let ctx = Context::new();
        assert_eq!(ctx.evaluate(&Token::NumberLiteral(42f32)).unwrap(),
                   Some(Value::Num(42f32)));
    }

    #[test]
    fn evaluate_handles_boolean_literals() {
        let ctx = Context::new();
        assert_eq!(ctx.evaluate(&Token::BooleanLiteral(true)).unwrap(),
                   Some(Value::Bool(true)));

        assert_eq!(ctx.evaluate(&Token::BooleanLiteral(false)).unwrap(),
                   Some(Value::Bool(false)));
    }

    #[test]
    fn evaluate_handles_identifiers() {
        let mut ctx = Context::new();
        ctx.set_val("var0", Value::Num(42f32));
        assert_eq!(ctx.evaluate(&Token::Identifier("var0".to_owned())).unwrap(),
                   Some(Value::Num(42f32)));
        assert_eq!(ctx.evaluate(&Token::Identifier("nope".to_owned())).unwrap(),
                   None);
    }

    #[test]
    fn evaluate_returns_none_on_invalid_token() {
        let ctx = Context::new();
        assert!(ctx.evaluate(&Token::DotDot).is_err());
    }
}
