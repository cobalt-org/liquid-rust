use crate::error::Error;
use crate::error::Result;
use crate::model::{Object, ObjectView, ScalarCow, Value, ValueCow, ValueView};

use super::Registers;

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

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.to_kstr();
        let data = &self.data;
        if data.contains_key(key.as_str()) {
            crate::model::try_find(data.as_value(), path)
        } else {
            self.parent.try_get(path)
        }
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let key = path.first().ok_or_else(|| {
            Error::with_msg("Unknown variable").context("requested variable", "nil")
        })?;
        let key = key.to_kstr();
        let data = &self.data;
        if data.contains_key(key.as_str()) {
            crate::model::find(data.as_value(), path).map(|v| v.into_owned().into())
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

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn registers(&self) -> &super::Registers {
        self.parent.registers()
    }

    fn render_mode(&self) -> &super::RenderingMode {
        self.parent.render_mode()
    }
}

/// A stack frame that only provides a sandboxed set of globals
pub struct GlobalFrame<P> {
    parent: P,
    data: std::cell::RefCell<Object>,
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

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            crate::model::try_find(data.as_value(), path).map(|v| v.into_owned().into())
        } else {
            self.parent.try_get(path)
        }
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let key = path.first().ok_or_else(|| {
            Error::with_msg("Unknown variable").context("requested variable", "nil")
        })?;
        let key = key.to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            crate::model::find(data.as_value(), path).map(|v| v.into_owned().into())
        } else {
            self.parent.get(path)
        }
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        let mut data = self.data.borrow_mut();
        data.insert(name, val)
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn registers(&self) -> &super::Registers {
        self.parent.registers()
    }

    fn render_mode(&self) -> &super::RenderingMode {
        self.parent.render_mode()
    }
}

pub(crate) struct IndexFrame<P> {
    parent: P,
    data: std::cell::RefCell<Object>,
}

impl<P: super::Runtime> IndexFrame<P> {
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

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            crate::model::try_find(data.as_value(), path).map(|v| v.into_owned().into())
        } else {
            self.parent.try_get(path)
        }
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let key = path.first().ok_or_else(|| {
            Error::with_msg("Unknown variable").context("requested variable", "nil")
        })?;
        let key = key.to_kstr();
        let data = self.data.borrow();
        if data.contains_key(key.as_str()) {
            crate::model::find(data.as_value(), path).map(|v| v.into_owned().into())
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

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        let mut data = self.data.borrow_mut();
        data.insert(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.data.borrow().get(name).map(|v| v.to_value().into())
    }

    fn registers(&self) -> &super::Registers {
        self.parent.registers()
    }

    fn render_mode(&self) -> &super::RenderingMode {
        self.parent.render_mode()
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
        Self {
            parent,
            name: None,
            data,
            registers: Default::default(),
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

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let key = path.first()?;
        let key = key.to_kstr();
        let data = &self.data;
        data.get(key.as_str())
            .and_then(|_| crate::model::try_find(data.as_value(), path))
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let key = path.first().ok_or_else(|| {
            Error::with_msg("Unknown variable").context("requested variable", "nil")
        })?;
        let key = key.to_kstr();
        let data = &self.data;
        data.get(key.as_str())
            .and_then(|_| crate::model::try_find(data.as_value(), path))
            .map(|v| v.into_owned().into())
            .ok_or_else(|| Error::with_msg("Unknown variable").context("requested variable", key))
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        self.parent.set_global(name, val)
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        self.parent.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.parent.get_index(name)
    }

    fn registers(&self) -> &super::Registers {
        &self.registers
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
}
