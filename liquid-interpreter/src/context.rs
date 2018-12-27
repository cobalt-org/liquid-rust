use std::sync;

use anymap;
use liquid_error::Error;
use liquid_error::Result;

use super::PartialStore;
use super::Renderable;
use super::Stack;
use super::ValueStore;

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

#[derive(Copy, Clone, Debug)]
struct NullPartials;

impl PartialStore for NullPartials {
    fn contains(&self, _name: &str) -> bool {
        false
    }

    fn names(&self) -> Vec<&str> {
        Vec::new()
    }

    fn try_get(&self, _name: &str) -> Option<sync::Arc<Renderable>> {
        None
    }

    fn get(&self, name: &str) -> Result<sync::Arc<Renderable>> {
        Err(Error::with_msg("Partial does not exist").context("name", name.to_owned()))
    }
}

/// Create processing context for a template.
pub struct ContextBuilder<'g> {
    globals: Option<&'g ValueStore>,
    partials: Option<&'g PartialStore>,
}

impl<'g> ContextBuilder<'g> {
    /// Creates a new, empty rendering context.
    pub fn new() -> Self {
        Self {
            globals: None,
            partials: None,
        }
    }

    /// Initialize the stack with the given globals.
    pub fn set_globals(mut self, values: &'g ValueStore) -> Self {
        self.globals = Some(values);
        self
    }

    /// Initialize partial-templates availible for including.
    pub fn set_partials(mut self, values: &'g PartialStore) -> Self {
        self.partials = Some(values);
        self
    }

    /// Create the `Context`.
    pub fn build(self) -> Context<'g> {
        let stack = match self.globals {
            Some(globals) => Stack::with_globals(globals),
            None => Stack::empty(),
        };
        let partials = self.partials.unwrap_or(&NullPartials);
        Context {
            stack,
            partials,
            registers: anymap::AnyMap::new(),
            interrupt: InterruptState::default(),
        }
    }
}

impl<'g> Default for ContextBuilder<'g> {
    fn default() -> Self {
        Self::new()
    }
}

/// Processing context for a template.
pub struct Context<'g> {
    stack: Stack<'g>,
    partials: &'g PartialStore,

    registers: anymap::AnyMap,
    interrupt: InterruptState,
}

impl<'g> Context<'g> {
    /// Create a default `Context`.
    ///
    /// See `ContextBuilder` for more control.
    pub fn new() -> Self {
        Context::default()
    }

    /// Access the block's `InterruptState`.
    pub fn interrupt(&self) -> &InterruptState {
        &self.interrupt
    }

    /// Access the block's `InterruptState`.
    pub fn interrupt_mut(&mut self) -> &mut InterruptState {
        &mut self.interrupt
    }

    /// Partial templates for inclusion.
    pub fn partials(&self) -> &PartialStore {
        self.partials
    }

    /// Data store for stateful tags/blocks.
    ///
    /// If a plugin needs state, it creates a `struct State : Default` and accesses it via
    /// `get_register_mut`.
    pub fn get_register_mut<T: anymap::any::IntoBox<anymap::any::Any> + Default>(
        &mut self,
    ) -> &mut T {
        self.registers.entry::<T>().or_insert_with(Default::default)
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

    /// Sets up a new stack frame, executes the supplied function and then
    /// tears the stack frame down before returning the function's result
    /// to the caller.
    pub fn run_in_named_scope<RvalT, S: Into<String>, FnT>(&mut self, name: S, f: FnT) -> RvalT
    where
        FnT: FnOnce(&mut Context) -> RvalT,
    {
        self.stack.push_named_frame(name);
        let result = f(self);
        self.stack.pop_frame();
        result
    }
}

impl<'g> Default for Context<'g> {
    fn default() -> Self {
        Self {
            stack: Stack::empty(),
            partials: &NullPartials,
            registers: anymap::AnyMap::new(),
            interrupt: InterruptState::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_value::Scalar;
    use liquid_value::Value;

    #[test]
    fn scoped_variables() {
        let test_path = [Scalar::new("test")];
        let global_path = [Scalar::new("global")];

        let mut ctx = Context::new();
        ctx.stack_mut().set_global("test", Value::scalar(42f64));
        assert_eq!(ctx.stack().get(&test_path).unwrap(), &Value::scalar(42f64));

        ctx.run_in_scope(|new_scope| {
            // assert that values are chained to the parent scope
            assert_eq!(
                new_scope.stack().get(&test_path).unwrap(),
                &Value::scalar(42f64)
            );

            // set a new local value, and assert that it overrides the previous value
            new_scope.stack_mut().set("test", Value::scalar(3.14f64));
            assert_eq!(
                new_scope.stack().get(&test_path).unwrap(),
                &Value::scalar(3.14f64)
            );

            // sat a new val that we will pick up outside the scope
            new_scope
                .stack_mut()
                .set_global("global", Value::scalar("some value"));
        });

        // assert that the value has reverted to the old one
        assert_eq!(ctx.stack().get(&test_path).unwrap(), &Value::scalar(42f64));
        assert_eq!(
            ctx.stack().get(&global_path).unwrap(),
            &Value::scalar("some value")
        );
    }
}
