use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{self, Arc, OnceLock};
#[cfg(feature = "conformance-harness")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

#[cfg(feature = "conformance-harness")]
use crate::conformance::FallbackFilterRegistry;
use crate::error::Error;
use crate::error::Result;
use crate::error::ResultLiquidExt;
#[cfg(feature = "conformance-harness")]
use crate::model::KString;
use crate::model::{
    ArrayView, DisplayCow, KStringCow, Object, ObjectView, PathElement, Scalar, State, Value,
    ValueCow, ValueView,
};
use crate::parser::{FilterCall, ParseFilter, PluginRegistry};

use super::PartialStore;
use super::Renderable;

/// State for rendering a template
pub trait Runtime {
    /// Partial templates for inclusion.
    fn partials(&self) -> &dyn PartialStore;

    /// The name of the currently active template.
    fn name(&self) -> Option<crate::model::KStringRef<'_>>;

    /// All available values
    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>>;
    /// Recursively index into the stack.
    fn try_get(&self, path: &[PathElement<'_>]) -> Option<ValueCow<'_>>;
    /// Recursively index into the stack.
    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>>;

    /// Sets a value in the global runtime.
    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value>;

    /// Sets a range binding in the global runtime.
    ///
    /// Implementors are responsible for choosing whether this preserves a lazy
    /// range binding or eagerly materializes it. The trait does not provide a
    /// default eager fallback.
    fn set_global_range(
        &self,
        name: crate::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<crate::model::Value>;

    /// Preserve a plain variable alias without eagerly materializing it.
    ///
    /// Returns true when the runtime handled the alias and no fallback
    /// `set_global` write is required.
    fn set_global_alias(
        &self,
        _name: crate::model::KString,
        _source: &[crate::model::PathElement<'_>],
    ) -> bool {
        false
    }

    /// Used by increment and decrement tags
    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value>;
    /// Used by increment and decrement tags
    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>>;

    /// Returns the bounds for a global range binding, when present.
    fn get_global_range_bounds(&self, _name: &str) -> Option<(i64, i64)> {
        None
    }

    /// Unnamed state for plugins during rendering
    fn registers(&self) -> &Registers;
}

/// Evaluate a filter call against a specific registry.
pub fn evaluate_filter_with_registry<R: Runtime + ?Sized>(
    runtime: &R,
    filter: &FilterCall,
    input: &dyn ValueView,
    filters: &PluginRegistry<Box<dyn ParseFilter>>,
) -> Result<Value> {
    let runtime_ref: &dyn Runtime = &runtime;
    let parser = filters.get(filter.name()).ok_or_else(|| {
        let mut available: Vec<_> = filters.plugin_names().collect();
        available.sort_unstable();
        let available = itertools::join(available, ", ");
        Error::with_msg("Unknown filter")
            .context("requested filter", filter.name().to_owned())
            .context("available filters", available)
    })?;

    let filter_impl = parser
        .parse(filter.args())
        .trace("Filter parsing error")
        .context_key("filter")
        .value_with(|| format!("{}", filter).into())?;

    filter_impl.evaluate(input, runtime_ref)
}

impl<R: Runtime + ?Sized> Runtime for &R {
    fn partials(&self) -> &dyn super::PartialStore {
        (**self).partials()
    }

    fn name(&self) -> Option<crate::model::KStringRef<'_>> {
        (**self).name()
    }

    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>> {
        (**self).roots()
    }

    fn try_get(&self, path: &[PathElement<'_>]) -> Option<ValueCow<'_>> {
        (**self).try_get(path)
    }

    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>> {
        (**self).get(path)
    }

    fn set_global(
        &self,
        name: crate::model::KString,
        val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        (**self).set_global(name, val)
    }

    fn set_global_range(
        &self,
        name: crate::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<crate::model::Value> {
        (**self).set_global_range(name, start, stop)
    }

    fn set_global_alias(
        &self,
        name: crate::model::KString,
        source: &[crate::model::PathElement<'_>],
    ) -> bool {
        (**self).set_global_alias(name, source)
    }

    fn set_index(&self, name: crate::model::KString, val: Value) -> Option<Value> {
        (**self).set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        (**self).get_index(name)
    }

    fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
        (**self).get_global_range_bounds(name)
    }

    fn registers(&self) -> &super::Registers {
        (**self).registers()
    }
}

/// Create processing runtime for a template.
pub struct RuntimeBuilder<'g, 'p> {
    globals: Option<&'g dyn ObjectView>,
    partials: Option<&'p dyn PartialStore>,
}

impl<'c, 'g: 'c, 'p: 'c> RuntimeBuilder<'g, 'p> {
    /// Creates a new, empty rendering runtime.
    pub fn new() -> Self {
        Self {
            globals: None,
            partials: None,
        }
    }

    /// Initialize the stack with the given globals.
    pub fn set_globals<'n>(self, values: &'n dyn ObjectView) -> RuntimeBuilder<'n, 'p> {
        RuntimeBuilder {
            globals: Some(values),
            partials: self.partials,
        }
    }

    /// Initialize partial-templates available for including.
    pub fn set_partials<'n>(self, values: &'n dyn PartialStore) -> RuntimeBuilder<'g, 'n> {
        RuntimeBuilder {
            globals: self.globals,
            partials: Some(values),
        }
    }

    /// Create the `Runtime`.
    pub fn build(self) -> impl Runtime + 'c {
        let partials = self.partials.unwrap_or(&NullPartials);
        let runtime = RuntimeCore {
            partials,
            ..Default::default()
        };
        // Order matters: this wraps caller-provided globals first so ordinary variable
        // lookups still flow through a normal stack frame.
        let runtime = super::StackFrame::new(runtime, self.globals.unwrap_or(&NullObject));
        // Then place the isolated increment/decrement counter store on top. Swapping
        // these two lines would reverse lookup precedence and let caller globals win
        // over counter values, which would break increment/decrement semantics. For
        // example:
        // `{% assign val = 9 %}{% increment val %}{{ val }}` => `09`
        // `{% decrement port %} {{ port }}` with `port: 10` => `-1 -1`
        let runtime = super::IndexFrame::new(runtime);
        super::GlobalFrame::new(runtime)
    }
}

#[derive(Copy, Clone, Debug)]
struct NullObject;

impl ValueView for NullObject {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> crate::model::DisplayCow<'_> {
        Value::Nil.render()
    }
    fn source(&self) -> crate::model::DisplayCow<'_> {
        Value::Nil.source()
    }
    fn type_name(&self) -> &'static str {
        "object"
    }
    fn query_state(&self, state: crate::model::State) -> bool {
        match state {
            crate::model::State::Truthy => true,
            crate::model::State::DefaultValue
            | crate::model::State::Empty
            | crate::model::State::Blank => false,
        }
    }

    fn to_kstr(&self) -> crate::model::KStringCow<'_> {
        crate::model::KStringCow::from_static("")
    }
    fn to_value(&self) -> Value {
        Value::Object(Object::new())
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for NullObject {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        0
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = crate::model::KStringCow<'k>> + 'k> {
        let keys = Vec::new().into_iter();
        Box::new(keys)
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let i = Vec::new().into_iter();
        Box::new(i)
    }

    fn iter<'k>(
        &'k self,
    ) -> Box<dyn Iterator<Item = (crate::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        let i = Vec::new().into_iter();
        Box::new(i)
    }

    fn contains_key(&self, _index: &str) -> bool {
        false
    }

    fn get<'s>(&'s self, _index: &str) -> Option<&'s dyn ValueView> {
        None
    }
}

impl Default for RuntimeBuilder<'static, 'static> {
    fn default() -> Self {
        Self::new()
    }
}

/// Processing runtime for a template.
pub struct RuntimeCore<'g> {
    partials: &'g dyn PartialStore,

    registers: Registers,
}

impl RuntimeCore<'_> {
    /// Create a default `RuntimeCore`.
    ///
    /// See `RuntimeBuilder` for more control.
    pub fn new() -> Self {
        RuntimeCore::default()
    }

    /// Partial templates for inclusion.
    pub fn partials(&self) -> &dyn PartialStore {
        self.partials
    }
}

impl Runtime for RuntimeCore<'_> {
    fn partials(&self) -> &dyn PartialStore {
        self.partials
    }

    fn name(&self) -> Option<crate::model::KStringRef<'_>> {
        None
    }

    fn roots(&self) -> std::collections::BTreeSet<crate::model::KStringCow<'_>> {
        // Indexes don't count
        std::collections::BTreeSet::new()
    }

    fn try_get(&self, _path: &[PathElement<'_>]) -> Option<ValueCow<'_>> {
        None
    }

    fn get(&self, path: &[PathElement<'_>]) -> Result<ValueCow<'_>> {
        let key = path
            .first()
            .map(|index| index.value().clone())
            .unwrap_or_else(|| Scalar::new("nil"));
        if !super::strict_variables_enabled(self) {
            return Ok(ValueCow::Owned(Value::Nil));
        }
        Error::with_msg("Unknown variable")
            .context("requested variable", key.to_kstr())
            .into_err()
    }

    fn set_global(
        &self,
        _name: crate::model::KString,
        _val: crate::model::Value,
    ) -> Option<crate::model::Value> {
        unreachable!("Must be masked by a global frame");
    }

    fn set_global_range(
        &self,
        _name: crate::model::KString,
        _start: i64,
        _stop: i64,
    ) -> Option<crate::model::Value> {
        unreachable!("Must be masked by a global frame");
    }

    fn set_index(&self, _name: crate::model::KString, _val: Value) -> Option<Value> {
        unreachable!("Must be masked by a global frame");
    }

    fn get_index<'a>(&'a self, _name: &str) -> Option<ValueCow<'a>> {
        None
    }

    fn registers(&self) -> &Registers {
        &self.registers
    }
}

impl Default for RuntimeCore<'_> {
    fn default() -> Self {
        Self {
            partials: &NullPartials,
            registers: Default::default(),
        }
    }
}

/// Unnamed state for plugins during rendering
pub struct Registers {
    registers: RefCell<anymap2::AnyMap>,
    #[cfg(feature = "conformance-harness")]
    live_scope_session: RefCell<Option<LiveScopeSession>>,
}

impl Registers {
    /// Data store for stateful tags/blocks.
    ///
    /// If a plugin needs state, it creates a `struct Register : Default` and accesses it via
    /// `get_mut`.
    pub fn get_mut<T: std::any::Any + Default>(&self) -> std::cell::RefMut<'_, T> {
        std::cell::RefMut::map(self.registers.borrow_mut(), |registers| {
            registers.entry::<T>().or_default()
        })
    }

    /// Return the active live-scope session, if present.
    #[cfg(feature = "conformance-harness")]
    pub fn live_scope_session(&self) -> Option<LiveScopeSession> {
        self.live_scope_session.borrow().clone()
    }

    /// Replace the active live-scope session.
    #[cfg(feature = "conformance-harness")]
    pub fn set_live_scope_session(&self, session: Option<LiveScopeSession>) {
        *self.live_scope_session.borrow_mut() = session;
    }

    /// Return whether the current render is inside a `{% render %}` partial boundary.
    pub fn in_render_tag_scope(&self) -> bool {
        self.get_mut::<RenderTagScopeRegister>().depth > 0
    }

    /// Enter a `{% render %}` partial boundary until the returned guard is dropped.
    pub fn enter_render_tag_scope(&self) -> RenderTagScopeGuard<'_> {
        self.get_mut::<RenderTagScopeRegister>().depth += 1;
        RenderTagScopeGuard { registers: self }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            registers: RefCell::new(anymap2::AnyMap::new()),
            #[cfg(feature = "conformance-harness")]
            live_scope_session: RefCell::new(None),
        }
    }
}

/// Hidden builder for conformance live-scope bindings.
#[doc(hidden)]
#[derive(Default)]
pub struct LiveScopeFrame {
    #[cfg(feature = "conformance-harness")]
    snapshot: LiveScopeSnapshot,
}

impl LiveScopeFrame {
    /// Create an empty live-scope frame.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a binding that should be visible to the conformance harness.
    pub fn insert<K: Into<crate::model::KString>>(&mut self, name: K, value: &dyn ValueView) {
        #[cfg(feature = "conformance-harness")]
        self.snapshot.insert(name, value);

        #[cfg(not(feature = "conformance-harness"))]
        {
            let _ = name;
            let _ = value;
        }
    }
}

/// Hidden guard for a pushed conformance live-scope frame.
#[doc(hidden)]
#[derive(Default)]
pub struct LiveScopeFrameGuard {
    #[cfg(feature = "conformance-harness")]
    _inner: Option<LiveScopeSessionGuard>,
}

/// Push a live-scope frame for the duration of the returned guard.
#[doc(hidden)]
pub fn push_live_scope_frame(runtime: &dyn Runtime, frame: LiveScopeFrame) -> LiveScopeFrameGuard {
    #[cfg(feature = "conformance-harness")]
    {
        LiveScopeFrameGuard {
            _inner: runtime
                .registers()
                .live_scope_session()
                .map(|session| session.push_root_scope(frame.snapshot)),
        }
    }

    #[cfg(not(feature = "conformance-harness"))]
    {
        let _ = runtime;
        let _ = frame;
        LiveScopeFrameGuard::default()
    }
}

#[derive(Default)]
struct RenderTagScopeRegister {
    depth: usize,
}

/// Guard tracking entry into a `{% render %}` partial boundary.
pub struct RenderTagScopeGuard<'a> {
    registers: &'a Registers,
}

impl Drop for RenderTagScopeGuard<'_> {
    fn drop(&mut self) {
        let mut register = self.registers.get_mut::<RenderTagScopeRegister>();
        register.depth = register.depth.saturating_sub(1);
    }
}

/// Shared handle to the active render policy for the current render tree.
#[derive(Clone, Default)]
pub(crate) struct ActivePolicyRegister {
    policy: Option<super::SharedRenderPolicy>,
}

impl ActivePolicyRegister {
    /// Install the active policy handle.
    pub(crate) fn set(&mut self, policy: Option<super::SharedRenderPolicy>) {
        self.policy = policy;
    }

    /// Return the active policy handle, if present.
    pub(crate) fn get(&self) -> Option<super::SharedRenderPolicy> {
        self.policy.as_ref().map(Rc::clone)
    }
}

/// Feature-gated register carrying the late-bound filter registry for conformance renders.
#[cfg(feature = "conformance-harness")]
#[derive(Clone, Default)]
pub(crate) struct FallbackFilterRegistryRegister {
    registry: Option<FallbackFilterRegistry>,
}

#[cfg(feature = "conformance-harness")]
impl FallbackFilterRegistryRegister {
    /// Install the active fallback filter registry.
    pub(crate) fn set(&mut self, registry: Option<FallbackFilterRegistry>) {
        self.registry = registry;
    }

    /// Return the active fallback filter registry.
    pub(crate) fn get(&self) -> Option<FallbackFilterRegistry> {
        self.registry.as_ref().map(Rc::clone)
    }
}

/// Shared render-local storage for Rust-only live root scopes.
#[cfg(feature = "conformance-harness")]
#[derive(Clone, Debug, Default)]
pub struct LiveScopeSession {
    inner: Arc<LiveScopeSessionInner>,
}

#[cfg(feature = "conformance-harness")]
#[derive(Debug, Default)]
struct LiveScopeSessionInner {
    active: AtomicBool,
    scopes: Mutex<Vec<LiveScopeSnapshot>>,
}

/// Owned snapshot of a Rust-only live root scope.
#[cfg(feature = "conformance-harness")]
#[derive(Clone, Debug, Default)]
pub struct LiveScopeSnapshot {
    entries: Object,
}

/// Owned live value returned from a snapshot lookup.
#[cfg(feature = "conformance-harness")]
#[derive(Clone, Debug)]
pub struct LiveScopeValue {
    value: Value,
}

#[cfg(feature = "conformance-harness")]
impl LiveScopeSnapshot {
    /// Create an empty live scope snapshot.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a live root binding into the snapshot.
    pub fn insert<K: Into<KString>>(&mut self, name: K, value: &dyn ValueView) {
        self.entries
            .insert(name.into(), value.to_live_scope_value());
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.entries.get(name)
    }
}

#[cfg(feature = "conformance-harness")]
impl LiveScopeValue {
    fn new(value: Value) -> Self {
        Self { value }
    }
}

#[cfg(feature = "conformance-harness")]
impl ValueView for LiveScopeValue {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self.value.as_debug()
    }

    fn render(&self) -> DisplayCow<'_> {
        self.value.render()
    }

    fn source(&self) -> DisplayCow<'_> {
        self.value.source()
    }

    fn type_name(&self) -> &'static str {
        self.value.type_name()
    }

    fn query_state(&self, state: State) -> bool {
        self.value.query_state(state)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        self.value.to_kstr()
    }

    fn to_value(&self) -> Value {
        self.value.to_value()
    }

    fn as_scalar(&self) -> Option<crate::model::ScalarCow<'_>> {
        self.value.as_scalar()
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        self.value.as_array()
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        self.value.as_object()
    }

    fn as_state(&self) -> Option<State> {
        self.value.as_state()
    }

    fn is_nil(&self) -> bool {
        self.value.is_nil()
    }
}

#[cfg(feature = "conformance-harness")]
impl LiveScopeSession {
    /// Create an active render-local session.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(LiveScopeSessionInner {
                active: AtomicBool::new(true),
                scopes: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Whether this session still belongs to an active render tree.
    pub fn is_active(&self) -> bool {
        self.inner.active.load(Ordering::SeqCst)
    }

    /// Mark the session inert after the owning render tree completes.
    pub fn deactivate(&self) {
        self.inner.active.store(false, Ordering::SeqCst);
    }

    pub(crate) fn shares_identity(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// Push a live root scope for the duration of a render frame.
    pub fn push_root_scope(&self, scope: LiveScopeSnapshot) -> LiveScopeSessionGuard {
        self.inner
            .scopes
            .lock()
            .expect("live scope mutex poisoned")
            .push(scope);
        LiveScopeSessionGuard {
            session: self.clone(),
        }
    }

    /// Look up a live root binding by name.
    pub fn find_root(&self, name: &str) -> Option<LiveScopeValue> {
        if !self.is_active() {
            return None;
        }

        self.inner
            .scopes
            .lock()
            .expect("live scope mutex poisoned")
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned().map(LiveScopeValue::new))
    }

    /// Return the current live scope depth.
    pub fn depth(&self) -> usize {
        if !self.is_active() {
            0
        } else {
            self.inner
                .scopes
                .lock()
                .expect("live scope mutex poisoned")
                .len()
        }
    }
}

/// Guard that pops the most recent live scope snapshot on drop.
#[cfg(feature = "conformance-harness")]
#[derive(Debug)]
pub struct LiveScopeSessionGuard {
    session: LiveScopeSession,
}

#[cfg(feature = "conformance-harness")]
impl Drop for LiveScopeSessionGuard {
    fn drop(&mut self) {
        self.session
            .inner
            .scopes
            .lock()
            .expect("live scope mutex poisoned")
            .pop();
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "conformance-harness")]
    use crate::model::ValueView;
    #[cfg(feature = "conformance-harness")]
    use crate::runtime::{Runtime, RuntimeBuilder};

    use super::RenderedBytesRegister;
    #[cfg(feature = "conformance-harness")]
    use super::{
        push_live_scope_frame, LiveScopeFrame, LiveScopeSession, LiveScopeSnapshot, Value,
    };

    #[cfg(feature = "conformance-harness")]
    #[test]
    fn live_scope_session_prefers_innermost_scope_and_pops_on_drop() {
        let session = LiveScopeSession::new();

        let mut outer = LiveScopeSnapshot::new();
        outer.insert("item", &Value::scalar("outer"));
        let outer_guard = session.push_root_scope(outer);

        assert_eq!(
            session
                .find_root("item")
                .map(|value| value.to_kstr().into_owned()),
            Some("outer".into())
        );
        assert_eq!(session.depth(), 1);

        {
            let mut inner = LiveScopeSnapshot::new();
            inner.insert("item", &Value::scalar("inner"));
            inner.insert("other", &Value::scalar("value"));
            let _inner_guard = session.push_root_scope(inner);

            assert_eq!(
                session
                    .find_root("item")
                    .map(|value| value.to_kstr().into_owned()),
                Some("inner".into())
            );
            assert_eq!(
                session
                    .find_root("other")
                    .map(|value| value.to_kstr().into_owned()),
                Some("value".into())
            );
            assert_eq!(session.depth(), 2);
        }

        assert_eq!(
            session
                .find_root("item")
                .map(|value| value.to_kstr().into_owned()),
            Some("outer".into())
        );
        assert!(session.find_root("other").is_none());
        assert_eq!(session.depth(), 1);

        drop(outer_guard);
        assert!(session.find_root("item").is_none());
        assert_eq!(session.depth(), 0);
    }

    #[cfg(feature = "conformance-harness")]
    #[test]
    fn live_scope_session_becomes_inert_after_deactivate() {
        let session = LiveScopeSession::new();

        let mut scope = LiveScopeSnapshot::new();
        scope.insert("item", &Value::scalar("outer"));
        let _guard = session.push_root_scope(scope);

        assert!(session.is_active());
        assert!(session.find_root("item").is_some());

        session.deactivate();

        assert!(!session.is_active());
        assert!(session.find_root("item").is_none());
        assert_eq!(session.depth(), 0);
    }

    #[cfg(feature = "conformance-harness")]
    #[test]
    fn push_live_scope_frame_uses_active_runtime_session() {
        let runtime = RuntimeBuilder::new().build();
        let session = LiveScopeSession::new();
        runtime
            .registers()
            .set_live_scope_session(Some(session.clone()));

        let mut frame = LiveScopeFrame::new();
        frame.insert("item", &Value::scalar("outer"));
        let _guard = push_live_scope_frame(&runtime, frame);

        assert_eq!(
            session
                .find_root("item")
                .map(|value| value.to_kstr().into_owned()),
            Some("outer".into())
        );
        assert_eq!(session.depth(), 1);
    }

    #[test]
    fn rendered_bytes_register_tracks_unreported_deltas_and_scope_resets() {
        let mut register = RenderedBytesRegister::default();

        register.add(5);
        assert_eq!(register.take_unreported_bytes(register.bytes()), 5);

        register.add(3);
        assert_eq!(register.take_unreported_bytes(register.bytes()), 3);

        register.add(4);
        register.clear_reported_baseline();
        assert_eq!(register.bytes(), 12);
        assert_eq!(register.take_unreported_bytes(register.bytes()), 0);

        register.add(2);
        assert_eq!(register.take_unreported_bytes(register.bytes()), 2);

        register.reset();
        assert_eq!(register.bytes(), 0);
        assert_eq!(register.take_unreported_bytes(register.bytes()), 0);
    }
}

/// Tracks the number of bytes written to the active rendered output buffer.
#[derive(Debug, Default)]
pub struct RenderedBytesRegister {
    bytes: usize,
    last_reported_bytes: usize,
}

impl RenderedBytesRegister {
    /// Add newly written bytes to the running total.
    pub fn add(&mut self, amount: usize) {
        self.bytes += amount;
    }

    /// Return the total number of bytes written so far.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// Return newly written bytes since the last resource-limit check and advance the baseline.
    pub fn take_unreported_bytes(&mut self, current_total: usize) -> usize {
        let delta = current_total.saturating_sub(self.last_reported_bytes);
        self.last_reported_bytes = current_total;
        delta
    }

    /// Clear the per-scope reporting baseline while preserving the cumulative byte total.
    pub fn clear_reported_baseline(&mut self) {
        self.last_reported_bytes = self.bytes;
    }

    /// Clear the running byte count before a fresh render begins.
    pub fn reset(&mut self) {
        self.bytes = 0;
        self.last_reported_bytes = 0;
    }
}

/// Internal representation of a global binding.
#[derive(Clone, Debug)]
pub(crate) enum GlobalBinding {
    Value(Value),
    Range(Arc<AssignedRangeValue>),
}

impl GlobalBinding {
    pub(crate) fn value(value: Value) -> Self {
        Self::Value(value)
    }

    pub(crate) fn range(start: i64, stop: i64) -> Self {
        Self::Range(Arc::new(AssignedRangeValue::new(start, stop)))
    }

    pub(crate) fn as_view(&self) -> &dyn ValueView {
        match self {
            Self::Value(value) => value.as_view(),
            Self::Range(range) => range.as_ref(),
        }
    }

    pub(crate) fn range_arc(&self) -> Option<Arc<AssignedRangeValue>> {
        match self {
            Self::Range(range) => Some(Arc::clone(range)),
            Self::Value(_) => None,
        }
    }

    pub(crate) fn range_bounds(&self) -> Option<(i64, i64)> {
        match self {
            Self::Range(range) => Some(range.bounds()),
            Self::Value(_) => None,
        }
    }
}

/// Array-like view for assigned range literals.
#[derive(Debug)]
pub(crate) struct AssignedRangeValue {
    start: i64,
    stop: i64,
    values: OnceLock<Vec<Value>>,
}

impl Clone for AssignedRangeValue {
    fn clone(&self) -> Self {
        let values = OnceLock::new();
        if let Some(existing) = self.values.get() {
            let _ = values.set(existing.clone());
        }

        Self {
            start: self.start,
            stop: self.stop,
            values,
        }
    }
}

impl AssignedRangeValue {
    /// Create a renderable range view.
    pub(crate) fn new(start: i64, stop: i64) -> Self {
        Self {
            start,
            stop,
            values: OnceLock::new(),
        }
    }

    fn size_hint(&self) -> i64 {
        if self.stop < self.start {
            0
        } else {
            self.stop.saturating_sub(self.start).saturating_add(1)
        }
    }

    fn normalized_index(&self, index: i64) -> Option<usize> {
        let size = self.size_hint();
        let index = if index >= 0 { index } else { size + index };

        if (0..size).contains(&index) {
            Some(index as usize)
        } else {
            None
        }
    }

    fn materialized_values(&self) -> &[Value] {
        self.values.get_or_init(|| {
            if self.stop < self.start {
                Vec::new()
            } else {
                (self.start..=self.stop).map(Value::scalar).collect()
            }
        })
    }

    pub(crate) fn bounds(&self) -> (i64, i64) {
        (self.start, self.stop)
    }

    #[cfg(test)]
    pub(crate) fn is_materialized(&self) -> bool {
        self.values.get().is_some()
    }
}

impl ValueView for AssignedRangeValue {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(format!("{}..{}", self.start, self.stop)))
    }

    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(format!("({}..{})", self.start, self.stop)))
    }

    fn type_name(&self) -> &'static str {
        "array"
    }

    fn query_state(&self, state: State) -> bool {
        match state {
            State::Truthy => true,
            State::DefaultValue | State::Empty | State::Blank => self.size_hint() == 0,
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        KStringCow::from_string(format!("{}..{}", self.start, self.stop))
    }

    fn to_value(&self) -> Value {
        Value::Array(self.materialized_values().to_vec())
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        Some(self)
    }
}

impl ArrayView for AssignedRangeValue {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.size_hint()
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        Box::new(
            self.materialized_values()
                .iter()
                .map(|value| value.as_view()),
        )
    }

    fn contains_key(&self, index: i64) -> bool {
        self.normalized_index(index).is_some()
    }

    fn get(&self, index: i64) -> Option<&dyn ValueView> {
        self.materialized_values()
            .get(self.normalized_index(index)?)
            .map(|value| value.as_view())
    }
}

/// The current interrupt state. The interrupt state is used by
/// the `break` and `continue` tags to halt template rendering
/// at a given point and unwind the `render` call stack until
/// it reaches an enclosing `for_loop`. At that point the interrupt
/// is cleared, and the `for_loop` carries on processing as directed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InterruptRegister {
    interrupt: Option<Interrupt>,
}

impl InterruptRegister {
    /// An interrupt state is active.
    pub fn interrupted(&self) -> bool {
        self.interrupt.is_some()
    }

    /// Sets the interrupt state. Any previous state is obliterated.
    pub fn set(&mut self, interrupt: Interrupt) {
        self.interrupt.replace(interrupt);
    }

    /// Fetches and clears the interrupt state.
    pub fn reset(&mut self) -> Option<Interrupt> {
        self.interrupt.take()
    }
}

/// Block processing interrupt state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    /// Restart processing the current block.
    Continue,
    /// Stop processing the current block.
    Break,
}

#[derive(Copy, Clone, Debug)]
struct NullPartials;

impl PartialStore for NullPartials {
    fn names(&self) -> Vec<&str> {
        Vec::new()
    }

    fn get(&self, _name: &str) -> Result<Option<sync::Arc<dyn Renderable>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::Scalar;
    use crate::model::Value;
    use crate::model::ValueViewCmp;

    #[test]
    fn mask_variables() {
        let test_path = [crate::model::PathElement::from(Scalar::new("test"))];

        let rt = RuntimeBuilder::new().build();
        rt.set_global("test".into(), Value::scalar(42f64));
        assert_eq!(&rt.get(&test_path).unwrap(), &ValueViewCmp::new(&42f64));

        {
            let data = crate::object!({"test": 3});
            let new_scope = super::super::StackFrame::new(&rt, &data);

            // assert that values are chained to the parent scope
            assert_eq!(&new_scope.get(&test_path).unwrap(), &ValueViewCmp::new(&3));
        }

        // assert that the value has reverted to the old one
        assert_eq!(&rt.get(&test_path).unwrap(), &ValueViewCmp::new(&42));
    }

    #[test]
    fn global_variables() {
        let global_path = [crate::model::PathElement::from(Scalar::new("global"))];

        let rt = RuntimeBuilder::new().build();

        {
            let data = crate::object!({"test": 3});
            let new_scope = super::super::StackFrame::new(&rt, &data);

            // sat a new val that we will pick up outside the scope
            new_scope.set_global("global".into(), Value::scalar("some value"));
        }
        assert_eq!(
            &rt.get(&global_path).unwrap(),
            &ValueViewCmp::new(&"some value")
        );
    }

    #[test]
    fn assigned_range_value_materializes_lazily() {
        let value = AssignedRangeValue::new(1, 3);

        assert!(value.values.get().is_none());
        assert_eq!(value.size(), 3);
        assert!(value.values.get().is_none());
        assert_eq!(value.render().to_string(), "1..3");
        assert!(value.values.get().is_none());

        assert_eq!(
            value.get(0).map(|item| item.to_kstr().into_owned()),
            Some("1".into())
        );
        assert!(value.values.get().is_some());
    }
}
