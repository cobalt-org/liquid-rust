use std::collections::HashMap;

use error::{Result, Error};
use value::{Value, Object, Index};

use super::Argument;
use super::{BoxedValueFilter, FilterValue};


pub fn unexpected_value_error<S: ToString>(expected: &str, actual: Option<S>) -> Error {
    let actual = actual.map(|x| x.to_string());
    unexpected_value_error_string(expected, actual)
}

pub fn unexpected_value_error_string(expected: &str, actual: Option<String>) -> Error {
    let actual = actual.unwrap_or_else(|| "nothing".to_owned());
    Error::with_msg(format!("Expected {}, found `{}`", expected, actual))
}

#[derive(Clone, Debug)]
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
    filters: HashMap<&'static str, BoxedValueFilter>,
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

    pub fn with_filters(mut self, filters: HashMap<&'static str, BoxedValueFilter>) -> Self {
        self.filters = filters;
        self
    }

    pub fn cycle_element(&mut self, name: &str, values: &[Argument]) -> Result<Option<Value>> {
        let index = {
            let i = self.cycles.entry(name.to_owned()).or_insert(0);
            let j = *i;
            *i = (*i + 1) % values.len();
            j
        };

        if index >= values.len() {
            return Err(Error::with_msg("cycle index out of bounds")
                           .context("index", &index)
                           .context("count", &values.len()));
        }

        let val = values[index].evaluate(self)?;
        Ok(Some(val))
    }

    pub fn add_filter(&mut self, name: &'static str, filter: BoxedValueFilter) {
        self.filters.insert(name, filter);
    }

    pub fn get_filter<'b>(&'b self, name: &str) -> Option<&'b FilterValue> {
        self.filters.get(name).map(|f| {
                                       let f: &FilterValue = f;
                                       f
                                   })
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

    /// Gets a value from the rendering context.
    pub fn get_val<'a>(&'a self, name: &str) -> Option<&'a Value> {
        for frame in self.stack.iter().rev() {
            if let rval @ Some(_) = frame.get(name) {
                return rval;
            }
        }
        self.globals.get(name)
    }

    pub fn get_val_by_index<'i, I: Iterator<Item = &'i Index>>(&self,
                                                               mut indexes: I)
                                                               -> Result<&Value> {
        let key = indexes
            .next()
            .ok_or_else(|| Error::with_msg("No index provided"))?;
        let key = key.as_key()
            .ok_or_else(|| {
                            Error::with_msg("Root index must be an object key")
                                .context("index", &key)
                        })?;
        let value = self.get_val(key)
            .ok_or_else(|| Error::with_msg("Invalid index").context("index", &key))?;

        indexes.fold(Ok(value), |value, index| {
            let value = value?;
            let child = value.get(index);
            let child =
                child
                    .ok_or_else(|| Error::with_msg("Invalid index").context("index", &key))?;
            Ok(child)
        })
    }

    /// Sets a value in the global context.
    pub fn set_global_val(&mut self, name: &str, val: Value) -> Option<Value> {
        self.globals.insert(name.to_owned(), val)
    }

    /// Sets a value to the rendering context.
    /// Note that it needs to be wrapped in a liquid::Value.
    ///
    /// # Panics
    ///
    /// Panics if there is no frame on the local values stack. Context
    /// instances are created with a top-level stack frame in place, so
    /// this should never happen in a well-formed program.
    pub fn set_val(&mut self, name: &str, val: Value) -> Option<Value> {
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
        ctx.set_global_val("number", Value::scalar(42f32));
        assert_eq!(ctx.get_val("number").unwrap(), &Value::scalar(42f32));
    }

    #[test]
    fn get_val_failure() {
        let mut ctx = Context::new();
        let mut post = Object::new();
        post.insert("number".to_owned(), Value::scalar(42f32));
        ctx.set_global_val("post", Value::Object(post));
        assert!(ctx.get_val("post.number").is_none());
    }

    #[test]
    fn get_val_by_index() {
        let mut ctx = Context::new();
        let mut post = Object::new();
        post.insert("number".to_owned(), Value::scalar(42f32));
        ctx.set_global_val("post", Value::Object(post));
        let indexes = vec![Index::with_key("post"), Index::with_key("number")];
        assert_eq!(ctx.get_val_by_index(indexes.iter()).unwrap(),
                   &Value::scalar(42f32));
    }

    #[test]
    fn scoped_variables() {
        let mut ctx = Context::new();
        ctx.set_global_val("test", Value::scalar(42f32));
        assert_eq!(ctx.get_val("test").unwrap(), &Value::scalar(42f32));

        ctx.run_in_scope(|new_scope| {
            // assert that values are chained to the parent scope
            assert_eq!(new_scope.get_val("test").unwrap(), &Value::scalar(42f32));

            // set a new local value, and assert that it overrides the previous value
            new_scope.set_val("test", Value::scalar(3.14f32));
            assert_eq!(new_scope.get_val("test").unwrap(), &Value::scalar(3.14f32));

            // sat a new val that we will pick up outside the scope
            new_scope.set_global_val("global", Value::scalar("some value"));
        });

        // assert that the value has reverted to the old one
        assert_eq!(ctx.get_val("test").unwrap(), &Value::scalar(42f32));
        assert_eq!(ctx.get_val("global").unwrap(), &Value::scalar("some value"));
    }
}
