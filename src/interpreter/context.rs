use std::collections::HashMap;

use error::{Result, Error};
use value::{Value, Object, Index};

use super::Argument;
use super::{BoxedValueFilter, FilterValue};


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
    filters: HashMap<String, BoxedValueFilter>,
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

    pub fn with_filters(mut self, filters: HashMap<String, BoxedValueFilter>) -> Self {
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
            return Err(Error::Render(format!("cycle index {} out of bounds {}",
                                             index,
                                             values.len())));
        }

        let val = values[index].evaluate(self)?;
        Ok(Some(val))
    }

    pub fn add_filter(&mut self, name: &str, filter: BoxedValueFilter) {
        self.filters.insert(name.to_owned(), filter);
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

    pub fn get_val_by_index<'v, 'i, I: Iterator<Item = &'i Index>>(&'v self,
                                                                   mut indexes: I)
                                                                   -> Result<&'v Value> {
        let key = indexes
            .next()
            .ok_or_else(|| Error::Render("No index provided".to_owned()))?;
        let key = key.as_key()
            .ok_or_else(|| {
                            Error::Render(format!("Root index must be an object key, found {:?}",
                                                  key))
                        })?;
        let value = self.get_val(key)
            .ok_or_else(|| Error::Render(format!("Object key not found: {:?}", key)))?;

        indexes.fold(Ok(value), |value, index| {
            let value = value?;
            let child = value.get(index);
            let child = child
                .ok_or_else(|| {
                                Error::Render(format!("Invalid index `{}` for value `{:?}`",
                                                      index,
                                                      value))
                            })?;
            Ok(child)
        })
    }

    /// Sets a value in the global context.
    pub fn set_val(&mut self, name: &str, val: Value) -> Option<Value> {
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
}
