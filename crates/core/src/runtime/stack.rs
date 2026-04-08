use crate::error::Error;
use crate::error::Result;
use crate::model::{Object, ObjectView, PathElement, Value, ValueCow, ValueView};
use std::collections::HashMap;

#[cfg(feature = "conformance-harness")]
use super::FallbackFilterRegistryRegister;
use super::{ActivePolicyRegister, GlobalBinding, Registers};

/// Layer variables on top of the existing runtime
pub struct StackFrame<P, O> {
    parent: P,
    name: Option<crate::model::KString>,
    data: O,
}

impl<P: super::Runtime, O: ObjectView> StackFrame<P, O> {
    /// Layer variables on top of the existing runtime
    pub fn new(parent: P, data: O) -> Self {
        Self {
            parent,
            name: None,
            data,
        }
    }

    /// Name the current context
    pub fn with_name<S: Into<crate::model::KString>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl<P: super::Runtime, O: ObjectView> super::Runtime for StackFrame<P, O> {
    fn partials(&self) -> &dyn super::PartialStore {
        self.parent.partials()
    }

    fn name(&self) -> Option<crate::model::KStringRef<'_>> {
        self.name
            .as_ref()
            .map(|n| n.as_ref())
            .or_else(|| self.parent.name())
    }

    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>> {
        let mut roots = self.parent.roots();
        roots.extend(self.data.keys());
        roots
    }

    fn try_get(&self, path: &[PathElement<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.value().to_kstr();
        let data = &self.data;
        if data.contains_key(key.as_str()) {
            crate::model::try_find(data.as_value(), path)
        } else {
            self.parent.try_get(path)
        }
    }

    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>> {
        let Some(key) = path.first() else {
            return missing_variable_or_nil(self, "nil");
        };
        let key = key.value().to_kstr();
        let data = &self.data;
        if data.contains_key(key.as_str()) {
            find_value_or_nil(self, data.as_value(), path)
        } else {
            self.parent.get(path)
        }
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        self.parent.set_global(name, val)
    }

    fn set_global_range(
        &self,
        name: crate::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<crate::model::Value> {
        self.parent.set_global_range(name, start, stop)
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
        self.parent.get_global_range_bounds(name)
    }

    fn registers(&self) -> &super::Registers {
        self.parent.registers()
    }
}

/// A stack frame that only provides a sandboxed set of globals
pub struct GlobalFrame<P> {
    parent: P,
    data: std::cell::RefCell<HashMap<crate::model::KString, GlobalBinding>>,
}

impl<P: super::Runtime> GlobalFrame<P> {
    /// Override globals for `parent`
    pub fn new(parent: P) -> Self {
        Self {
            parent,
            data: Default::default(),
        }
    }
}

impl<P: super::Runtime> super::Runtime for GlobalFrame<P> {
    fn partials(&self) -> &dyn super::PartialStore {
        self.parent.partials()
    }

    fn name(&self) -> Option<crate::model::KStringRef<'_>> {
        self.parent.name()
    }

    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>> {
        let mut roots = self.parent.roots();
        roots.extend(self.data.borrow().keys().map(|k| k.clone().into()));
        roots
    }

    fn try_get(&self, path: &[PathElement<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.value().to_kstr();
        let data = self.data.borrow();
        match data.get(key.as_str()) {
            Some(GlobalBinding::Value(value)) => {
                crate::model::try_find(value.as_view(), &path[1..]).map(|v| v.into_owned().into())
            }
            Some(binding) => {
                if path.len() == 1 {
                    binding.range_arc().map(|range| ValueCow::Shared(range))
                } else {
                    crate::model::try_find(binding.as_view(), &path[1..])
                        .map(|v| v.into_owned().into())
                }
            }
            None => self.parent.try_get(path),
        }
    }

    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>> {
        let Some(key) = path.first() else {
            return missing_variable_or_nil(self, "nil");
        };
        let key = key.value().to_kstr();
        let data = self.data.borrow();
        match data.get(key.as_str()) {
            Some(GlobalBinding::Value(value)) => {
                find_value_or_nil(self, value.as_view(), &path[1..])
                    .map(|value| value.into_owned().into())
            }
            Some(binding) => {
                if path.len() == 1 {
                    Ok(ValueCow::Shared(
                        binding
                            .range_arc()
                            .expect("range binding should provide range value"),
                    ))
                } else {
                    find_value_or_nil(self, binding.as_view(), &path[1..])
                        .map(|value| value.into_owned().into())
                }
            }
            None => self.parent.get(path),
        }
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        let mut data = self.data.borrow_mut();
        match data.insert(name, GlobalBinding::value(val)) {
            Some(GlobalBinding::Value(value)) => Some(value),
            Some(GlobalBinding::Range(_)) | None => None,
        }
    }

    fn set_global_range(
        &self,
        name: crate::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<crate::model::Value> {
        let mut data = self.data.borrow_mut();
        match data.insert(name, GlobalBinding::range(start, stop)) {
            Some(GlobalBinding::Value(value)) => Some(value),
            Some(GlobalBinding::Range(_)) | None => None,
        }
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
        match self.data.borrow().get(name) {
            Some(binding) => binding.range_bounds(),
            None => self.parent.get_global_range_bounds(name),
        }
    }

    fn registers(&self) -> &super::Registers {
        self.parent.registers()
    }
}

/// A stack frame with an isolated increment/decrement index store.
///
/// This layer intentionally sits above normal caller globals on lookup so counter
/// values win once a name has been incremented or decremented.
pub struct IndexFrame<P> {
    parent: P,
    data: std::cell::RefCell<Object>,
}

impl<P: super::Runtime> IndexFrame<P> {
    /// Create a new isolated index frame layered over `parent`.
    pub fn new(parent: P) -> Self {
        Self {
            parent,
            data: Default::default(),
        }
    }
}

impl<P: super::Runtime> super::Runtime for IndexFrame<P> {
    fn partials(&self) -> &dyn super::PartialStore {
        self.parent.partials()
    }

    fn name(&self) -> Option<crate::model::KStringRef<'_>> {
        self.parent.name()
    }

    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>> {
        let mut roots = self.parent.roots();
        roots.extend(self.data.borrow().keys().map(|k| k.clone().into()));
        roots
    }

    fn try_get(&self, path: &[PathElement<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.value().to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            crate::model::try_find(data.as_value(), path).map(|v| v.into_owned().into())
        } else {
            self.parent.try_get(path)
        }
    }

    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>> {
        let Some(key) = path.first() else {
            return missing_variable_or_nil(self, "nil");
        };
        let key = key.value().to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            find_value_or_nil(self, data.as_value(), path).map(|value| value.into_owned().into())
        } else {
            self.parent.get(path)
        }
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        self.parent.set_global(name, val)
    }

    fn set_global_range(
        &self,
        name: crate::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<crate::model::Value> {
        self.parent.set_global_range(name, start, stop)
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        let mut data = self.data.borrow_mut();
        data.insert(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.data.borrow().get(name).map(|v| v.to_value().into())
    }

    fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
        self.parent.get_global_range_bounds(name)
    }

    fn registers(&self) -> &super::Registers {
        self.parent.registers()
    }
}

/// A [`StackFrame`] where variables are not recursively searched for,
/// However, you can still access the parent's partials.
pub struct SandboxedStackFrame<P, O> {
    parent: P,
    name: Option<crate::model::KString>,
    data: O,
    registers: Registers,
}

impl<P: super::Runtime, O: ObjectView> SandboxedStackFrame<P, O> {
    /// Create a new [`SandboxedStackFrame`] from a parent and some data
    pub fn new(parent: P, data: O) -> Self {
        let registers = Registers::default();
        #[cfg(feature = "conformance-harness")]
        registers.set_live_scope_session(parent.registers().live_scope_session());
        if let Some(policy) = parent.registers().get_mut::<ActivePolicyRegister>().get() {
            registers
                .get_mut::<ActivePolicyRegister>()
                .set(Some(policy));
        }
        #[cfg(feature = "conformance-harness")]
        if let Some(fallback_filters) = parent
            .registers()
            .get_mut::<FallbackFilterRegistryRegister>()
            .get()
        {
            registers
                .get_mut::<FallbackFilterRegistryRegister>()
                .set(Some(fallback_filters));
        }
        Self {
            parent,
            name: None,
            data,
            registers,
        }
    }

    /// Name the current context
    pub fn with_name<S: Into<crate::model::KString>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl<P: super::Runtime, O: ObjectView> super::Runtime for SandboxedStackFrame<P, O> {
    fn partials(&self) -> &dyn super::PartialStore {
        self.parent.partials()
    }

    fn name(&self) -> Option<crate::model::KStringRef<'_>> {
        self.name
            .as_ref()
            .map(|n| n.as_ref())
            .or_else(|| self.parent.name())
    }

    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>> {
        let mut roots = std::collections::BTreeSet::new();
        roots.extend(self.data.keys());
        roots
    }

    fn try_get(&self, path: &[PathElement<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.value().to_kstr();
        let data = &self.data;
        data.get(key.as_str())
            .and_then(|_| crate::model::try_find(data.as_value(), path))
    }

    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>> {
        let Some(key) = path.first() else {
            return missing_variable_or_nil(self, "nil");
        };
        let key = key.value().to_kstr();
        let data = &self.data;
        if data.get(key.as_str()).is_some() {
            find_value_or_nil(self, data.as_value(), path)
        } else {
            missing_variable_or_nil(self, key)
        }
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        self.parent.set_global(name, val)
    }

    fn set_global_range(
        &self,
        name: crate::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<crate::model::Value> {
        self.parent.set_global_range(name, start, stop)
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
        self.parent.get_global_range_bounds(name)
    }

    fn registers(&self) -> &super::Registers {
        &self.registers
    }
}

fn find_value_or_nil<'o>(
    runtime: &dyn super::Runtime,
    value: &'o dyn ValueView,
    path: &[PathElement<'_>],
) -> Result<ValueCow<'o>> {
    if super::strict_variables_enabled(runtime) {
        crate::model::find(value, path)
    } else {
        Ok(crate::model::try_find(value, path).unwrap_or_else(|| ValueCow::Owned(Value::Nil)))
    }
}

fn missing_variable_or_nil(
    runtime: &dyn super::Runtime,
    requested: impl Into<crate::model::KString>,
) -> Result<ValueCow<'static>> {
    if super::strict_variables_enabled(runtime) {
        Error::with_msg("Unknown variable")
            .context("requested variable", requested.into())
            .into_err()
    } else {
        Ok(ValueCow::Owned(Value::Nil))
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::RuntimeBuilder, Runtime};

    use super::*;

    #[test]
    fn test_opaque_stack_frame_try_get() {
        let globals = {
            let mut o = Object::new();
            o.insert("a".into(), Value::Scalar(1i64.into()));
            o
        };
        let runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let opaque_stack_frame = SandboxedStackFrame::new(&runtime, {
            let mut o = Object::new();
            o.insert("b".into(), Value::Scalar(2i64.into()));
            o
        });

        // Testing that you can access variables in the current frame, but not the parent
        assert!(opaque_stack_frame.try_get(&["a".into()]).is_none());
        assert!(opaque_stack_frame.try_get(&["b".into()]).is_some());

        let stack_frame = StackFrame::new(opaque_stack_frame, {
            let mut o = Object::new();
            o.insert("c".into(), Value::Scalar(1i64.into()));
            o
        });

        // Testing that a child of a OpaqueStackFrame can access access OpaqueStackFrame's variables but not the parent
        assert!(stack_frame.try_get(&["a".into()]).is_none());
        assert!(stack_frame.try_get(&["b".into()]).is_some());
        assert!(stack_frame.try_get(&["c".into()]).is_some());
    }

    #[test]
    fn test_opaque_stack_frame_get() {
        let globals = {
            let mut o = Object::new();
            o.insert("a".into(), Value::Scalar(1i64.into()));
            o
        };
        let runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let opaque_stack_frame = SandboxedStackFrame::new(&runtime, {
            let mut o = Object::new();
            o.insert("b".into(), Value::Scalar(2i64.into()));
            o
        });

        // Testing that you can access variables in the current frame, but not the parent
        assert!(opaque_stack_frame.get(&["a".into()]).is_err());
        assert!(opaque_stack_frame.get(&["b".into()]).is_ok());

        let stack_frame = StackFrame::new(opaque_stack_frame, {
            let mut o = Object::new();
            o.insert("c".into(), Value::Scalar(1i64.into()));
            o
        });

        // Testing that a child of a OpaqueStackFrame can access access OpaqueStackFrame's variables but not the parent
        assert!(stack_frame.get(&["a".into()]).is_err());
        assert!(stack_frame.get(&["b".into()]).is_ok());
        assert!(stack_frame.get(&["c".into()]).is_ok());
    }

    #[test]
    fn test_opaque_stack_frame_roots() {
        let globals = {
            let mut o = Object::new();
            o.insert("a".into(), Value::Scalar(1i64.into()));
            o
        };
        let runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let opaque_stack_frame = SandboxedStackFrame::new(&runtime, {
            let mut o = Object::new();
            o.insert("b".into(), Value::Scalar(2i64.into()));
            o
        });
        let roots = opaque_stack_frame.roots();

        // Testing that the roots are not copied from the parent
        assert!(!roots.contains("a"));
        assert!(roots.contains("b"));
    }

    #[cfg(feature = "conformance-harness")]
    #[test]
    fn sandboxed_stack_frame_copies_live_scope_session() {
        let globals = Object::new();
        let runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let session = super::super::LiveScopeSession::new();
        runtime
            .registers()
            .set_live_scope_session(Some(session.clone()));

        let opaque_stack_frame = SandboxedStackFrame::new(&runtime, Object::new());
        let inherited = opaque_stack_frame
            .registers()
            .live_scope_session()
            .expect("sandboxed frame should inherit active live scope session");

        assert!(inherited.is_active());

        let mut scope = super::super::LiveScopeSnapshot::new();
        scope.insert("item", &Value::scalar("value"));
        let _guard = session.push_root_scope(scope);

        assert_eq!(
            inherited
                .find_root("item")
                .map(|value| value.to_kstr().into_owned()),
            Some("value".into())
        );
    }

    #[test]
    fn overwriting_a_range_binding_keeps_the_old_range_lazy() {
        let runtime = RuntimeBuilder::new().build();
        let globals = GlobalFrame::new(runtime);

        assert!(globals
            .set_global_range("foo".into(), 1, 1_000_000)
            .is_none());

        let old_range = globals
            .data
            .borrow()
            .get("foo")
            .and_then(GlobalBinding::range_arc)
            .expect("range binding should be stored lazily");
        assert!(!old_range.is_materialized());

        assert!(globals.set_global("foo".into(), Value::Nil).is_none());
        assert!(!old_range.is_materialized());
        assert_eq!(
            globals
                .get(&[crate::model::Scalar::new("foo").into()])
                .unwrap()
                .to_value(),
            Value::Nil
        );
    }

    #[test]
    fn child_global_frame_inherits_ancestor_range_bounds() {
        let runtime = RuntimeBuilder::new().build();
        let parent = GlobalFrame::new(&runtime);
        let child = GlobalFrame::new(&parent);

        assert!(parent.set_global_range("foo".into(), 2, 5).is_none());
        assert_eq!(child.get_global_range_bounds("foo"), Some((2, 5)));
    }

    #[test]
    fn child_value_binding_suppresses_parent_range_bounds() {
        let runtime = RuntimeBuilder::new().build();
        let parent = GlobalFrame::new(&runtime);
        let child = GlobalFrame::new(&parent);

        assert!(parent.set_global_range("foo".into(), 2, 5).is_none());
        assert!(child.set_global("foo".into(), Value::scalar("shadow")).is_none());
        assert_eq!(child.get_global_range_bounds("foo"), None);
    }
}
