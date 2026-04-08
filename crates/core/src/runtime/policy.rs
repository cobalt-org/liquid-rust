use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "conformance-harness")]
use crate::conformance::ConformanceCallbacks;
use crate::error::Error;
use crate::error::Result;
use crate::model::Value;
use crate::model::ValueView;
use crate::parser::FilterCall;
use crate::parser::ParseFilter;
use crate::parser::PluginRegistry;

use super::evaluate_filter_with_registry;
use super::ActivePolicyRegister;
#[cfg(feature = "conformance-harness")]
use super::FallbackFilterRegistryRegister;
use super::RenderedBytesRegister;
use super::Runtime;

/// Shared handle to the active per-render policy implementation.
pub(crate) type SharedRenderPolicy = Rc<dyn RenderPolicy>;

mod private {
    pub trait Sealed {}
}

/// Internal render policy abstraction used by the executor and select runtime frames.
pub(crate) trait RenderPolicy: private::Sealed {
    /// Handle a render error and optionally provide replacement output.
    fn handle_render_error(&self, _runtime: &dyn Runtime, error: Error) -> Result<Option<String>> {
        Err(error)
    }

    /// Track render-op usage for the current render tree.
    fn increment_render_ops(&self, _amount: usize) -> Result<()> {
        Ok(())
    }

    /// Track assign-like writes for the current render tree.
    fn increment_assign_bytes(&self, _amount: usize) -> Result<()> {
        Ok(())
    }

    /// Compute the assign-like resource cost for the provided value.
    fn assign_resource_cost(&self, value: &Value) -> usize {
        prod_assign_resource_cost(value)
    }

    /// Check current render-resource limits.
    fn check_resource_limits(&self, _runtime: &dyn Runtime, _rendered_bytes: usize) -> Result<()> {
        Ok(())
    }

    /// Reset per-scope resource bookkeeping while preserving cumulative state.
    fn reset_resource_limits(&self) -> Result<()> {
        Ok(())
    }

    /// Whether missing variables should resolve leniently.
    fn strict_variables(&self) -> bool {
        true
    }

    /// Whether unknown filters should error instead of passing input through.
    fn strict_filters(&self) -> bool {
        true
    }

    /// Enter a nested partial/render boundary.
    fn enter_scope(&self) -> Result<()> {
        Ok(())
    }

    /// Leave a nested partial/render boundary.
    fn leave_scope(&self) {}

    /// Drain any collected lenient render errors.
    fn take_errors(&self) -> Vec<Error> {
        Vec::new()
    }
}

/// Hidden production render error mode used by the stable root facade.
#[derive(Clone, Copy, Debug)]
pub enum ProdErrorMode {
    /// Abort on the first render error.
    Strict,
    /// Format the error inline and collect the original value.
    Lenient(fn(&Error) -> String),
}

impl Default for ProdErrorMode {
    fn default() -> Self {
        Self::Lenient(default_error_formatter)
    }
}

fn default_error_formatter(error: &Error) -> String {
    error.to_string()
}

/// Hidden production render policy configuration shared between workspace crates.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProdPolicyConfig {
    /// Optional maximum render-op count across the full render tree.
    pub max_render_ops: Option<usize>,
    /// Optional maximum assign-byte count across the full render tree.
    pub max_assign_bytes: Option<usize>,
    /// Optional maximum nested partial/render depth.
    pub max_depth: Option<usize>,
    /// Whether missing variables should raise instead of resolving to nil.
    pub strict_variables: bool,
    /// Whether missing filters should raise instead of passing input through.
    pub strict_filters: bool,
    /// How runtime render errors should be surfaced.
    pub error_mode: ProdErrorMode,
}

#[derive(Debug, Default)]
struct ProdPolicyState {
    render_ops: usize,
    assign_bytes: usize,
    depth: usize,
    errors: Vec<Error>,
}

/// Production render policy state.
#[derive(Debug)]
pub(crate) struct ProdPolicy {
    config: ProdPolicyConfig,
    state: RefCell<ProdPolicyState>,
}

impl ProdPolicy {
    /// Create a shared production policy handle.
    pub(crate) fn shared(config: ProdPolicyConfig) -> SharedRenderPolicy {
        Rc::new(Self {
            config,
            state: RefCell::new(ProdPolicyState::default()),
        })
    }
}

impl private::Sealed for ProdPolicy {}

impl RenderPolicy for ProdPolicy {
    fn handle_render_error(&self, _runtime: &dyn Runtime, error: Error) -> Result<Option<String>> {
        match self.config.error_mode {
            ProdErrorMode::Strict => Err(error),
            ProdErrorMode::Lenient(formatter) => {
                self.state.borrow_mut().errors.push(error.clone());
                Ok(Some(formatter(&error)))
            }
        }
    }

    fn increment_render_ops(&self, amount: usize) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.render_ops += amount;
        enforce_limit("Render", self.config.max_render_ops, state.render_ops)
    }

    fn increment_assign_bytes(&self, amount: usize) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.assign_bytes += amount;
        enforce_limit("Assign", self.config.max_assign_bytes, state.assign_bytes)
    }

    fn strict_variables(&self) -> bool {
        self.config.strict_variables
    }

    fn strict_filters(&self) -> bool {
        self.config.strict_filters
    }

    fn enter_scope(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();
        let next_depth = state.depth + 1;
        enforce_limit("Depth", self.config.max_depth, next_depth)?;
        state.depth = next_depth;
        Ok(())
    }

    fn leave_scope(&self) {
        let mut state = self.state.borrow_mut();
        state.depth = state.depth.saturating_sub(1);
    }

    fn take_errors(&self) -> Vec<Error> {
        std::mem::take(&mut self.state.borrow_mut().errors)
    }
}

/// Feature-gated Ruby conformance policy scaffold.
#[cfg(feature = "conformance-harness")]
#[derive(Clone)]
pub(crate) struct ConformancePolicyConfig {
    pub(crate) strict_variables: bool,
    pub(crate) strict_filters: bool,
    pub(crate) callbacks: Rc<dyn ConformanceCallbacks>,
}

#[cfg(feature = "conformance-harness")]
#[derive(Default)]
struct ConformancePolicyState {
    depth: usize,
}

#[cfg(feature = "conformance-harness")]
pub(crate) struct RubyConformancePolicy {
    config: ConformancePolicyConfig,
    state: RefCell<ConformancePolicyState>,
}

#[cfg(feature = "conformance-harness")]
impl RubyConformancePolicy {
    const MAX_DEPTH: usize = 100;

    /// Create a shared conformance policy handle.
    pub(crate) fn shared(config: ConformancePolicyConfig) -> SharedRenderPolicy {
        Rc::new(Self {
            config,
            state: RefCell::new(ConformancePolicyState::default()),
        })
    }
}

#[cfg(feature = "conformance-harness")]
impl private::Sealed for RubyConformancePolicy {}

#[cfg(feature = "conformance-harness")]
impl RenderPolicy for RubyConformancePolicy {
    fn handle_render_error(&self, runtime: &dyn Runtime, error: Error) -> Result<Option<String>> {
        self.config.callbacks.handle_render_error(runtime, error)
    }

    fn increment_render_ops(&self, amount: usize) -> Result<()> {
        self.config.callbacks.increment_render_ops(amount)
    }

    fn increment_assign_bytes(&self, amount: usize) -> Result<()> {
        self.config.callbacks.increment_assign_bytes(amount)
    }

    fn assign_resource_cost(&self, value: &Value) -> usize {
        legacy_assign_score(value)
    }

    fn check_resource_limits(&self, runtime: &dyn Runtime, rendered_bytes: usize) -> Result<()> {
        self.config
            .callbacks
            .check_resource_limits(runtime, rendered_bytes)
    }

    fn reset_resource_limits(&self) -> Result<()> {
        self.config.callbacks.reset_resource_limits()
    }

    fn strict_variables(&self) -> bool {
        self.config.strict_variables
    }

    fn strict_filters(&self) -> bool {
        self.config.strict_filters
    }

    fn enter_scope(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();
        let next_depth = state.depth + 1;
        if next_depth > Self::MAX_DEPTH {
            return Error::with_msg("stack level too deep").into_err();
        }
        state.depth = next_depth;
        Ok(())
    }

    fn leave_scope(&self) {
        let mut state = self.state.borrow_mut();
        state.depth = state.depth.saturating_sub(1);
    }
}

fn active_policy(runtime: &dyn Runtime) -> Option<SharedRenderPolicy> {
    runtime.registers().get_mut::<ActivePolicyRegister>().get()
}

/// Install the active render policy into the runtime registers.
pub(crate) fn install_policy(runtime: &dyn Runtime, policy: SharedRenderPolicy) {
    runtime
        .registers()
        .get_mut::<ActivePolicyRegister>()
        .set(Some(policy));
}

/// Install the default production render policy into the runtime registers.
pub fn install_prod_policy(runtime: &dyn Runtime, config: ProdPolicyConfig) {
    install_policy(runtime, ProdPolicy::shared(config));
}

/// Install the conformance-harness render policy into the runtime registers.
#[cfg(feature = "conformance-harness")]
pub(crate) fn install_conformance_policy(runtime: &dyn Runtime, config: ConformancePolicyConfig) {
    install_policy(runtime, RubyConformancePolicy::shared(config));
}

/// Whether strict variable lookup is enabled for the current render tree.
pub(crate) fn strict_variables_enabled(runtime: &dyn Runtime) -> bool {
    active_policy(runtime)
        .map(|policy| policy.strict_variables())
        .unwrap_or(true)
}

/// Handle a render error via the active render policy.
pub(crate) fn handle_render_error(runtime: &dyn Runtime, error: Error) -> Result<Option<String>> {
    if let Some(policy) = active_policy(runtime) {
        policy.handle_render_error(runtime, error)
    } else {
        Err(error)
    }
}

/// Track render-op usage via the active render policy.
pub(crate) fn increment_render_ops(runtime: &dyn Runtime, amount: usize) -> Result<()> {
    if let Some(policy) = active_policy(runtime) {
        policy.increment_render_ops(amount)
    } else {
        Ok(())
    }
}

/// Track assign-byte usage via the active render policy.
pub fn increment_assign_bytes(runtime: &dyn Runtime, amount: usize) -> Result<()> {
    if let Some(policy) = active_policy(runtime) {
        policy.increment_assign_bytes(amount)
    } else {
        Ok(())
    }
}

/// Compute the assign resource cost for the active render policy.
pub fn assign_resource_cost(runtime: &dyn Runtime, value: &Value) -> usize {
    active_policy(runtime)
        .map(|policy| policy.assign_resource_cost(value))
        .unwrap_or_else(|| prod_assign_resource_cost(value))
}

/// Check resource limits via the active render policy.
pub(crate) fn check_resource_limits(runtime: &dyn Runtime) -> Result<()> {
    if let Some(policy) = active_policy(runtime) {
        let rendered_bytes = runtime
            .registers()
            .get_mut::<RenderedBytesRegister>()
            .bytes();
        policy.check_resource_limits(runtime, rendered_bytes)
    } else {
        Ok(())
    }
}

/// Reset per-scope resource limits via the active render policy.
pub fn reset_resource_limits(runtime: &dyn Runtime) -> Result<()> {
    if let Some(policy) = active_policy(runtime) {
        policy.reset_resource_limits()
    } else {
        Ok(())
    }
}

/// Enter a nested render/include boundary while preserving shared policy state.
pub fn enter_render_scope(runtime: &dyn Runtime) -> Result<RenderScopeGuard> {
    if let Some(policy) = active_policy(runtime) {
        policy.enter_scope()?;
        Ok(RenderScopeGuard {
            policy: Some(policy),
        })
    } else {
        Ok(RenderScopeGuard { policy: None })
    }
}

/// Drain collected production render errors for the current render tree.
pub fn take_render_errors(runtime: &dyn Runtime) -> Vec<Error> {
    active_policy(runtime)
        .map(|policy| policy.take_errors())
        .unwrap_or_default()
}

fn prod_assign_resource_cost(value: &Value) -> usize {
    match value {
        Value::Scalar(scalar) => scalar.clone().into_cow_str().len(),
        Value::Array(values) => 1 + values.iter().map(prod_assign_resource_cost).sum::<usize>(),
        Value::Object(values) => {
            1 + values
                .iter()
                .map(|(key, value)| key.as_str().len() + prod_assign_resource_cost(value))
                .sum::<usize>()
        }
        Value::State(_) | Value::Nil => 1,
    }
}

#[cfg_attr(not(feature = "conformance-harness"), allow(dead_code))]
fn legacy_assign_score(value: &Value) -> usize {
    match value {
        Value::Scalar(scalar) if scalar.is_string() => scalar.clone().into_cow_str().len(),
        Value::Array(values) => 1 + values.iter().map(legacy_assign_score).sum::<usize>(),
        Value::Object(values) => {
            1 + values
                .iter()
                .map(|(key, value)| key.as_str().len() + legacy_assign_score(value))
                .sum::<usize>()
        }
        Value::Scalar(_) | Value::State(_) | Value::Nil => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{legacy_assign_score, prod_assign_resource_cost};

    #[test]
    fn legacy_assign_score_matches_shopify_assign_semantics() {
        assert_eq!(1, legacy_assign_score(&value!(123)));
        assert_eq!(3, legacy_assign_score(&value!("123")));
        assert_eq!(1, legacy_assign_score(&value!([])));
        assert_eq!(2, legacy_assign_score(&value!([123])));
        assert_eq!(6, legacy_assign_score(&value!([123, "abcd"])));
        assert_eq!(1, legacy_assign_score(&value!({})));
        assert_eq!(5, legacy_assign_score(&value!({"int": 123})));
        assert_eq!(
            12,
            legacy_assign_score(&value!({"int": 123, "str": "abcd"}))
        );
    }

    #[test]
    fn production_assign_cost_preserves_current_scalar_string_lengths() {
        assert_eq!(3, prod_assign_resource_cost(&value!(123)));
        assert_eq!(3, prod_assign_resource_cost(&value!("123")));
    }
}

/// Evaluate a filter call using the parsed registry plus any conformance fallback registry.
pub(crate) fn evaluate_filter(
    runtime: &dyn Runtime,
    filter: &FilterCall,
    input: &dyn ValueView,
    parsed_filters: &PluginRegistry<Box<dyn ParseFilter>>,
) -> Result<Value> {
    #[cfg(feature = "conformance-harness")]
    {
        let fallback_filters = {
            runtime
                .registers()
                .get_mut::<FallbackFilterRegistryRegister>()
                .get()
        };
        if let Some(fallback_filters) = fallback_filters {
            if fallback_filters.has_filter(filter.name()) {
                return fallback_filters.evaluate(filter, input, runtime);
            }
        }
    }

    if parsed_filters.get(filter.name()).is_some() {
        return evaluate_filter_with_registry(runtime, filter, input, parsed_filters);
    }

    if active_policy(runtime)
        .map(|policy| policy.strict_filters())
        .unwrap_or(true)
    {
        evaluate_filter_with_registry(runtime, filter, input, parsed_filters)
    } else {
        Ok(input.to_value())
    }
}

fn enforce_limit(kind: &str, limit: Option<usize>, actual: usize) -> Result<()> {
    if let Some(limit) = limit {
        if actual > limit {
            return Error::with_msg(format!("{kind} limit exceeded"))
                .context("limit", limit.to_string())
                .context("actual", actual.to_string())
                .into_err();
        }
    }

    Ok(())
}

/// Guard that decrements depth tracking when a partial/render boundary exits.
pub struct RenderScopeGuard {
    policy: Option<SharedRenderPolicy>,
}

impl Drop for RenderScopeGuard {
    fn drop(&mut self) {
        if let Some(policy) = &self.policy {
            policy.leave_scope();
        }
    }
}
