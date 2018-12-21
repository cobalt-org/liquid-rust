use std::borrow;

use itertools;
use liquid_error::{Error, Result};
use liquid_value::{Object, PathRef, Scalar, Value};

use super::ValueStore;

#[derive(Clone, Default, Debug)]
struct Frame {
    name: Option<String>,
    data: Object,
}

impl Frame {
    fn new() -> Self {
        Default::default()
    }

    fn with_name<S: Into<String>>(name: S) -> Self {
        Self {
            name: Some(name.into()),
            data: Object::new(),
        }
    }
}

/// Stack of variables.
#[derive(Debug, Clone)]
pub struct Stack<'g> {
    globals: Option<&'g ValueStore>,
    stack: Vec<Frame>,
    // State of variables created through increment or decrement tags.
    indexes: Object,
}

impl<'g> Stack<'g> {
    /// Create an empty stack
    pub fn empty() -> Self {
        Self {
            globals: None,
            indexes: Object::new(),
            // Mutable frame for globals.
            stack: vec![Frame::new()],
        }
    }

    /// Create a stack initialized with read-only `ValueStore`.
    pub fn with_globals(globals: &'g ValueStore) -> Self {
        let mut stack = Self::empty();
        stack.globals = Some(globals);
        stack
    }

    /// Creates a new variable scope chained to a parent scope.
    pub(crate) fn push_frame(&mut self) {
        self.stack.push(Frame::new());
    }

    /// Creates a new variable scope chained to a parent scope.
    pub(crate) fn push_named_frame<S: Into<String>>(&mut self, name: S) {
        self.stack.push(Frame::with_name(name));
    }

    /// Removes the topmost stack frame from the local variable stack.
    ///
    /// # Panics
    ///
    /// This method will panic if popping the topmost frame results in an
    /// empty stack. Given that a context is created with a top-level stack
    /// frame already in place, emptying the stack should never happen in a
    /// well-formed program.
    pub(crate) fn pop_frame(&mut self) {
        if self.stack.pop().is_none() {
            panic!("Unbalanced push/pop, leaving the stack empty.")
        };
    }

    /// The name of the currently active template.
    pub fn frame_name(&self) -> Option<&str> {
        self.stack
            .iter()
            .rev()
            .find_map(|f| f.name.as_ref().map(|s| s.as_str()))
    }

    /// Recursively index into the stack.
    pub fn try_get(&self, path: PathRef) -> Option<&Value> {
        let frame = self.find_path_frame(path)?;

        frame.try_get_variable(path)
    }

    /// Recursively index into the stack.
    pub fn get(&self, path: PathRef) -> Result<&Value> {
        let frame = self.find_path_frame(path).ok_or_else(|| {
            let key = path
                .iter()
                .next()
                .cloned()
                .unwrap_or_else(|| Scalar::new("nil"));
            let globals = itertools::join(self.globals().iter(), ", ");
            Error::with_msg("Unknown variable")
                .context("requested variable", key.to_str().into_owned())
                .context("available variables", globals)
        })?;

        frame.get_variable(path)
    }

    fn globals(&self) -> Vec<&str> {
        let mut globals = self.globals.map(|g| g.roots()).unwrap_or_default();
        for frame in self.stack.iter() {
            globals.extend(frame.data.roots());
        }
        globals.sort();
        globals.dedup();
        globals
    }

    fn find_path_frame<'a>(&'a self, path: PathRef) -> Option<&'a ValueStore> {
        let key = path.iter().next()?;
        let key = key.to_str();
        self.find_frame(key.as_ref())
    }

    fn find_frame<'a>(&'a self, name: &str) -> Option<&'a ValueStore> {
        for frame in self.stack.iter().rev() {
            if frame.data.contains_root(name) {
                return Some(&frame.data);
            }
        }

        if self.globals.map(|g| g.contains_root(name)).unwrap_or(false) {
            return self.globals;
        }

        if self.indexes.contains_root(name) {
            return Some(&self.indexes);
        }

        None
    }

    /// Used by increment and decrement tags
    pub fn set_index<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<borrow::Cow<'static, str>>,
    {
        self.indexes.insert(name.into(), val)
    }

    /// Used by increment and decrement tags
    pub fn get_index<'a>(&'a self, name: &str) -> Option<&'a Value> {
        self.indexes.get(name)
    }

    /// Sets a value in the global context.
    pub fn set_global<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<borrow::Cow<'static, str>>,
    {
        self.global_frame().insert(name.into(), val)
    }

    /// Sets a value to the rendering context.
    /// Note that it needs to be wrapped in a liquid::Value.
    ///
    /// # Panics
    ///
    /// Panics if there is no frame on the local values stack. Context
    /// instances are created with a top-level stack frame in place, so
    /// this should never happen in a well-formed program.
    pub fn set<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<borrow::Cow<'static, str>>,
    {
        self.current_frame().insert(name.into(), val)
    }

    fn current_frame(&mut self) -> &mut Object {
        match self.stack.last_mut() {
            Some(frame) => &mut frame.data,
            None => panic!("Global frame removed."),
        }
    }

    fn global_frame(&mut self) -> &mut Object {
        match self.stack.first_mut() {
            Some(frame) => &mut frame.data,
            None => panic!("Global frame removed."),
        }
    }
}

impl<'g> Default for Stack<'g> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stack_find_frame() {
        let mut stack = Stack::empty();
        stack.set_global("number", Value::scalar(42f64));
        assert!(stack.find_frame("number").is_some(),);
    }

    #[test]
    fn stack_find_frame_failure() {
        let mut stack = Stack::empty();
        let mut post = Object::new();
        post.insert("number".into(), Value::scalar(42f64));
        stack.set_global("post", Value::Object(post));
        assert!(stack.find_frame("post.number").is_none());
    }

    #[test]
    fn stack_get() {
        let mut stack = Stack::empty();
        let mut post = Object::new();
        post.insert("number".into(), Value::scalar(42f64));
        stack.set_global("post", Value::Object(post));
        let indexes = [Scalar::new("post"), Scalar::new("number")];
        assert_eq!(stack.get(&indexes).unwrap(), &Value::scalar(42f64));
    }

}
