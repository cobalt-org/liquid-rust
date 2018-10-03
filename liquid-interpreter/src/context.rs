use std::borrow;
use std::collections::HashMap;
use std::sync;

use error::{Error, Result};
use value::{Index, Object, Value};

use super::Argument;
use super::Globals;
use super::{BoxedValueFilter, FilterValue};

/// Format an error for an unexpected value.
pub fn unexpected_value_error<S: ToString>(expected: &str, actual: Option<S>) -> Error {
    let actual = actual.map(|x| x.to_string());
    unexpected_value_error_string(expected, actual)
}

fn unexpected_value_error_string(expected: &str, actual: Option<String>) -> Error {
    let actual = actual.unwrap_or_else(|| "nothing".to_owned());
    Error::with_msg(format!("Expected {}, found `{}`", expected, actual))
}

/// Block processing interrupt state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    /// Restart processing the current block.
    Continue,
    /// Stop processing the current block.
    Break,
}

/// The current interrupt state. The interrupt state is used by
/// the `break` and `continue` tags to halt template rendering
/// at a given point and unwind the `render` call stack until
/// it reaches an enclosing `for_loop`. At that point the interrupt
/// is cleared, and the `for_loop` carries on processing as directed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InterruptState {
    interrupt: Option<Interrupt>,
}

impl InterruptState {
    /// An interrupt state is active.
    pub fn interrupted(&self) -> bool {
        self.interrupt.is_some()
    }

    /// Sets the interrupt state. Any previous state is obliterated.
    pub fn set_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt = Some(interrupt);
    }

    /// Fetches and clears the interrupt state.
    pub fn pop_interrupt(&mut self) -> Option<Interrupt> {
        let rval = self.interrupt;
        self.interrupt = None;
        rval
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct CycleStateInner {
    // The indices of all the cycles encountered during rendering.
    cycles: HashMap<String, usize>,
}

impl CycleStateInner {
    fn cycle_index(&mut self, name: &str, max: usize) -> usize {
        let i = self.cycles.entry(name.to_owned()).or_insert(0);
        let j = *i;
        *i = (*i + 1) % max;
        j
    }
}

/// See `cycle` tag.
pub struct CycleState<'a, 'g>
where
    'g: 'a,
{
    context: &'a mut Context<'g>,
}

impl<'a, 'g> CycleState<'a, 'g> {
    /// See `cycle` tag.
    pub fn cycle_element(&mut self, name: &str, values: &[Argument]) -> Result<Value> {
        let index = self.context.cycles.cycle_index(name, values.len());
        if index >= values.len() {
            return Err(Error::with_msg(
                "cycle index out of bounds, most likely from mismatched cycles",
            ).context("index", format!("{}", index))
                .context("count", format!("{}", values.len())));
        }

        let val = values[index].evaluate(self.context)?;
        Ok(val)
    }
}

/// Stack of variables.
#[derive(Debug, Clone)]
pub struct Stack<'g> {
    globals: Option<&'g Globals>,
    stack: Vec<Object>,
}

impl<'g> Stack<'g> {
    /// Create an empty stack
    pub fn empty() -> Self {
        Self {
            globals: None,
            // Mutable frame for globals.
            stack: vec![Object::new()],
        }
    }

    /// Create a stack initialized with read-only `Globals`.
    pub fn with_globals(globals: &'g Globals) -> Self {
        let mut stack = Self::empty();
        stack.globals = Some(globals);
        stack
    }

    /// Creates a new variable scope chained to a parent scope.
    fn push_frame(&mut self) {
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
    fn pop_frame(&mut self) {
        if self.stack.pop().is_none() {
            panic!("Pop leaves empty stack")
        };
    }

    /// Recursively index into the stack.
    pub fn get_val_by_index<'i, I: Iterator<Item = &'i Index>>(
        &self,
        mut indexes: I,
    ) -> Result<&Value> {
        let key = indexes
            .next()
            .ok_or_else(|| Error::with_msg("No index provided"))?;
        let key = key.as_key().ok_or_else(|| {
            Error::with_msg("Root index must be an object key").context("index", format!("{}", key))
        })?;
        let value = self.get_val(key)?;

        indexes.fold(Ok(value), |value, index| {
            let value = value?;
            let child = value.get(index);
            let child = child
                .ok_or_else(|| Error::with_msg("Invalid index").context("index", key.to_owned()))?;
            Ok(child)
        })
    }

    fn get_val<'a>(&'a self, name: &str) -> Result<&'a Value> {
        for frame in self.stack.iter().rev() {
            if let Some(rval) = frame.get(name) {
                return Ok(rval);
            }
        }
        self.globals
            .ok_or_else(|| Error::with_msg("Invalid index").context("index", name.to_owned()))
            .and_then(|g| g.get(name))
    }

    /// Sets a value in the global context.
    pub fn set_global_val<S>(&mut self, name: S, val: Value) -> Option<Value>
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
    pub fn set_val<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<borrow::Cow<'static, str>>,
    {
        self.current_frame().insert(name.into(), val)
    }

    fn current_frame(&mut self) -> &mut Object {
        match self.stack.last_mut() {
            Some(frame) => frame,
            None => panic!("Global frame removed."),
        }
    }

    fn global_frame(&mut self) -> &mut Object {
        match self.stack.first_mut() {
            Some(frame) => frame,
            None => panic!("Global frame removed."),
        }
    }
}

impl<'g> Default for Stack<'g> {
    fn default() -> Self {
        Self::empty()
    }
}

/// Create processing context for a template.
pub struct ContextBuilder<'g> {
    globals: Option<&'g Globals>,
    filters: sync::Arc<HashMap<&'static str, BoxedValueFilter>>,
}

impl<'g> ContextBuilder<'g> {
    /// Creates a new, empty rendering context.
    pub fn new() -> Self {
        Self {
            globals: None,
            filters: Default::default(),
        }
    }

    /// Initialize the stack with the given globals.
    pub fn set_globals(mut self, values: &'g Globals) -> Self {
        self.globals = Some(values);
        self
    }

    /// Initialize the context with the given filters.
    pub fn set_filters(
        mut self,
        filters: &sync::Arc<HashMap<&'static str, BoxedValueFilter>>,
    ) -> Self {
        self.filters = sync::Arc::clone(filters);
        self
    }

    /// Create the `Context`.
    pub fn build(self) -> Context<'g> {
        let stack = match self.globals {
            Some(globals) => Stack::with_globals(globals),
            None => Stack::empty(),
        };
        Context {
            stack,
            interrupt: InterruptState::default(),
            cycles: CycleStateInner::default(),
            filters: self.filters,
        }
    }
}

impl<'g> Default for ContextBuilder<'g> {
    fn default() -> Self {
        Self::new()
    }
}

/// Processing context for a template.
#[derive(Default)]
pub struct Context<'g> {
    stack: Stack<'g>,

    interrupt: InterruptState,
    cycles: CycleStateInner,

    filters: sync::Arc<HashMap<&'static str, BoxedValueFilter>>,
}

impl<'g> Context<'g> {
    /// Create a default `Context`.
    ///
    /// See `ContextBuilder` for more control.
    pub fn new() -> Self {
        Context::default()
    }

    /// Grab a `FilterValue`.
    pub fn get_filter<'b>(&'b self, name: &str) -> Option<&'b FilterValue> {
        self.filters.get(name).map(|f| {
            let f: &FilterValue = f;
            f
        })
    }

    /// Access the block's `InterruptState`.
    pub fn interrupt(&self) -> &InterruptState {
        &self.interrupt
    }

    /// Access the block's `InterruptState`.
    pub fn interrupt_mut(&mut self) -> &mut InterruptState {
        &mut self.interrupt
    }

    /// See `cycle` tag.
    pub fn cycles<'a>(&'a mut self) -> CycleState<'a, 'g>
    where
        'g: 'a,
    {
        CycleState { context: self }
    }

    /// Access the current `Stack`.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    /// Access the current `Stack`.
    pub fn stack_mut<'a>(&'a mut self) -> &'a mut Stack<'g>
    where
        'g: 'a,
    {
        &mut self.stack
    }

    /// Sets up a new stack frame, executes the supplied function and then
    /// tears the stack frame down before returning the function's result
    /// to the caller.
    pub fn run_in_scope<RvalT, FnT>(&mut self, f: FnT) -> RvalT
    where
        FnT: FnOnce(&mut Context) -> RvalT,
    {
        self.stack.push_frame();
        let result = f(self);
        self.stack.pop_frame();
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_val() {
        let mut ctx = Context::new();
        ctx.stack_mut()
            .set_global_val("number", Value::scalar(42f64));
        assert_eq!(
            ctx.stack().get_val("number").unwrap(),
            &Value::scalar(42f64)
        );
    }

    #[test]
    fn get_val_failure() {
        let mut ctx = Context::new();
        let mut post = Object::new();
        post.insert("number".into(), Value::scalar(42f64));
        ctx.stack_mut().set_global_val("post", Value::Object(post));
        assert!(ctx.stack().get_val("post.number").is_err());
    }

    #[test]
    fn get_val_by_index() {
        let mut ctx = Context::new();
        let mut post = Object::new();
        post.insert("number".into(), Value::scalar(42f64));
        ctx.stack_mut().set_global_val("post", Value::Object(post));
        let indexes = vec![Index::with_key("post"), Index::with_key("number")];
        assert_eq!(
            ctx.stack().get_val_by_index(indexes.iter()).unwrap(),
            &Value::scalar(42f64)
        );
    }

    #[test]
    fn scoped_variables() {
        let mut ctx = Context::new();
        ctx.stack_mut().set_global_val("test", Value::scalar(42f64));
        assert_eq!(ctx.stack().get_val("test").unwrap(), &Value::scalar(42f64));

        ctx.run_in_scope(|new_scope| {
            // assert that values are chained to the parent scope
            assert_eq!(
                new_scope.stack().get_val("test").unwrap(),
                &Value::scalar(42f64)
            );

            // set a new local value, and assert that it overrides the previous value
            new_scope
                .stack_mut()
                .set_val("test", Value::scalar(3.14f64));
            assert_eq!(
                new_scope.stack().get_val("test").unwrap(),
                &Value::scalar(3.14f64)
            );

            // sat a new val that we will pick up outside the scope
            new_scope
                .stack_mut()
                .set_global_val("global", Value::scalar("some value"));
        });

        // assert that the value has reverted to the old one
        assert_eq!(ctx.stack().get_val("test").unwrap(), &Value::scalar(42f64));
        assert_eq!(
            ctx.stack().get_val("global").unwrap(),
            &Value::scalar("some value")
        );
    }
}
