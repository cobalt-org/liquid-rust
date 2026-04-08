use liquid_core::{runtime::LiveScopeSession, ValueView};
use magnus::{
    r_array::RArray, r_hash::RHash, typed_data, value::ReprValue, Error as MagnusError, IntoValue,
    TryConvert, Value,
};

use crate::values;

pub(crate) const RENDER_SESSION_REF_KEY: &str = "__render_session_ref";

#[magnus::wrap(
    class = "Liquid::RustExtension::RenderSessionRef",
    free_immediately,
    size
)]
#[derive(Clone, Debug)]
pub(crate) struct RenderSessionRef {
    session: LiveScopeSession,
}

impl RenderSessionRef {
    pub(crate) fn new(session: LiveScopeSession) -> Self {
        Self { session }
    }

    pub(crate) fn session(&self) -> LiveScopeSession {
        self.session.clone()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.session.is_active()
    }
}

pub(crate) fn ext_context_new(
    ruby: &magnus::Ruby,
    scopes: Option<Value>,
    registers: Option<Value>,
    error_mode: Option<String>,
    parent_handle: Option<RHash>,
) -> Result<RHash, MagnusError> {
    let handle = ruby.hash_new();
    let scopes = scopes
        .and_then(RArray::from_value)
        .unwrap_or_else(|| ruby.ary_new());

    handle.aset("scopes", scopes)?;
    handle.aset(
        "registers",
        registers.unwrap_or_else(|| ruby.hash_new().as_value()),
    )?;
    handle.aset(
        "error_mode",
        error_mode.unwrap_or_else(|| "strict".to_string()),
    )?;

    if let Some(parent_handle) = parent_handle {
        if let Some(session_ref) = parent_handle.get(RENDER_SESSION_REF_KEY) {
            handle.aset(RENDER_SESSION_REF_KEY, session_ref)?;
        }
    }

    Ok(handle)
}

pub(crate) fn ext_context_set(
    ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
    value: Value,
) -> Result<Value, MagnusError> {
    let scopes: RArray = handle.lookup("scopes")?;
    let target: Value = if scopes.is_empty() {
        let hash = ruby.hash_new();
        scopes.push(hash)?;
        hash.as_value()
    } else {
        scopes.entry((scopes.len() - 1) as isize)?
    };

    let _: Value = target.funcall("[]=", (key, value))?;
    Ok(value)
}

pub(crate) fn ext_context_get(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<Value, MagnusError> {
    if let Some(value) = lookup_live_root(&handle, &key)? {
        return Ok(value);
    }

    lookup_scope_value(handle, &key)
}

pub(crate) fn ext_context_find_variable(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<Value, MagnusError> {
    if let Some(value) = lookup_live_root(&handle, &key)? {
        return Ok(value);
    }

    lookup_scope_value(handle, &key)
}

pub(crate) fn ext_context_find_live_root(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<Value, MagnusError> {
    Ok(lookup_live_root(&handle, &key)?.unwrap_or_else(|| {
        magnus::Ruby::get()
            .expect("Ruby VM should be available")
            .qnil()
            .as_value()
    }))
}

pub(crate) fn ext_context_has_live_root(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<bool, MagnusError> {
    Ok(active_render_session(handle)
        .and_then(|session| session.find_root(&key))
        .is_some())
}

pub(crate) fn ext_context_live_depth(
    _ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<usize, MagnusError> {
    Ok(active_render_session(handle).map_or(0, |session| session.depth()))
}

pub(crate) fn ext_context_has_key(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<bool, MagnusError> {
    if active_render_session(handle)
        .and_then(|session| session.find_root(&key))
        .is_some()
    {
        return Ok(true);
    }

    scope_contains_key(handle, &key)
}

pub(crate) fn ext_context_push(
    ruby: &magnus::Ruby,
    handle: RHash,
    new_scope: Option<RHash>,
) -> Result<(), MagnusError> {
    let scopes: RArray = handle.lookup("scopes")?;
    scopes.push(new_scope.unwrap_or_else(|| ruby.hash_new()))?;
    Ok(())
}

pub(crate) fn ext_context_pop(_ruby: &magnus::Ruby, handle: RHash) -> Result<Value, MagnusError> {
    let scopes: RArray = handle.lookup("scopes")?;
    scopes.pop()
}

pub(crate) fn active_render_session(handle: RHash) -> Option<LiveScopeSession> {
    render_session_ref(handle)
        .filter(|session_ref| session_ref.is_active())
        .map(|session_ref| session_ref.session())
}

pub(crate) fn set_render_session_ref(
    ruby: &magnus::Ruby,
    handle: RHash,
    session: LiveScopeSession,
) -> Result<(), MagnusError> {
    let session_ref = ruby
        .obj_wrap(RenderSessionRef::new(session))
        .into_value_with(ruby);
    handle.aset(RENDER_SESSION_REF_KEY, session_ref)?;
    Ok(())
}

pub(crate) fn clear_render_session_ref(handle: RHash) -> Result<(), MagnusError> {
    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    handle.aset(RENDER_SESSION_REF_KEY, ruby.qnil())?;
    Ok(())
}

fn render_session_ref(handle: RHash) -> Option<typed_data::Obj<RenderSessionRef>> {
    handle
        .get(RENDER_SESSION_REF_KEY)
        .and_then(|value| typed_data::Obj::<RenderSessionRef>::try_convert(value).ok())
}

fn lookup_live_root(handle: &RHash, key: &str) -> Result<Option<Value>, MagnusError> {
    let Some(session) = active_render_session(*handle) else {
        return Ok(None);
    };
    let Some(value) = session.find_root(key) else {
        return Ok(None);
    };

    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let value = value.to_value();
    values::liquid_model_to_ruby_value(&ruby, &value).map(Some)
}

fn lookup_scope_value(handle: RHash, key: &str) -> Result<Value, MagnusError> {
    let scopes: RArray = handle.lookup("scopes")?;
    for idx in (0..scopes.len()).rev() {
        let scope: Value = scopes.entry(idx as isize)?;
        if let Some(hash) = RHash::from_value(scope) {
            if let Some(value) = hash.get(key) {
                return resolve_scope_value(handle, hash.as_value(), key, value);
            }
            continue;
        }

        if scope.respond_to("key?", false)? {
            let has_key: bool = scope.funcall("key?", (key,))?;
            if !has_key {
                continue;
            }
        }

        if scope.respond_to("[]", false)? {
            let value: Value = scope.funcall("[]", (key,))?;
            return resolve_scope_value(handle, scope, key, value);
        }
    }

    Ok(magnus::Ruby::get()
        .expect("Ruby VM should be available")
        .qnil()
        .as_value())
}

fn scope_contains_key(handle: RHash, key: &str) -> Result<bool, MagnusError> {
    let scopes: RArray = handle.lookup("scopes")?;
    for idx in (0..scopes.len()).rev() {
        let scope: Value = scopes.entry(idx as isize)?;
        if let Some(hash) = RHash::from_value(scope) {
            if hash.get(key).is_some() {
                return Ok(true);
            }
            continue;
        }

        if scope.respond_to("key?", false)? {
            let has_key: bool = scope.funcall("key?", (key,))?;
            if has_key {
                return Ok(true);
            }
        } else if scope.respond_to("[]", false)? {
            let value: Value = scope.funcall("[]", (key,))?;
            if !value.is_nil() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn resolve_scope_value(
    handle: RHash,
    scope: Value,
    key: &str,
    value: Value,
) -> Result<Value, MagnusError> {
    if !ruby_value_is_proc(value) {
        return Ok(value);
    }

    let arity: i64 = value.funcall("arity", ())?;
    let resolved = if arity == 0 {
        value.funcall("call", ())?
    } else if let Some(context) = handle.get("context") {
        value.funcall("call", (context,))?
    } else {
        value.funcall("call", ())?
    };

    if let Some(hash) = RHash::from_value(scope) {
        hash.aset(key, resolved)?;
    } else if scope.respond_to("[]=", false)? {
        let _: Value = scope.funcall("[]=", (key, resolved))?;
    }

    Ok(resolved)
}

fn ruby_value_is_proc(value: Value) -> bool {
    let class_name = unsafe { value.classname() };
    class_name.as_ref() == "Proc"
}
