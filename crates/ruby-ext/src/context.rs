use magnus::{r_array::RArray, r_hash::RHash, value::ReprValue, Error as MagnusError, Value};

pub(crate) fn ext_context_new(
    ruby: &magnus::Ruby,
    environments: Option<Value>,
    registers: Option<Value>,
    error_mode: Option<String>,
) -> Result<RHash, MagnusError> {
    let handle = ruby.hash_new();
    let scopes = ruby.ary_new();

    if let Some(environments) = environments {
        if let Some(array) = RArray::from_value(environments) {
            for idx in 0..array.len() {
                let scope: Value = array.entry(idx as isize)?;
                scopes.push(scope)?;
            }
        } else {
            scopes.push(environments)?;
        }
    }

    handle.aset("scopes", scopes)?;
    handle.aset("registers", registers.unwrap_or_else(|| ruby.hash_new().as_value()))?;
    handle.aset("error_mode", error_mode.unwrap_or_else(|| "strict".to_string()))?;
    Ok(handle)
}

pub(crate) fn ext_context_set(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
    value: Value,
) -> Result<Value, MagnusError> {
    let scopes: RArray = handle.lookup("scopes")?;
    let target = if scopes.is_empty() {
        let hash = magnus::Ruby::get().expect("Ruby VM should be available").hash_new();
        scopes.push(hash)?;
        hash
    } else {
        scopes.entry::<RHash>((scopes.len() - 1) as isize)?
    };

    target.aset(key, value)?;
    Ok(value)
}

pub(crate) fn ext_context_get(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<Value, MagnusError> {
    lookup_scope_value(handle, &key)
}

pub(crate) fn ext_context_find_variable(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<Value, MagnusError> {
    lookup_scope_value(handle, &key)
}

pub(crate) fn ext_context_has_key(
    _ruby: &magnus::Ruby,
    handle: RHash,
    key: String,
) -> Result<bool, MagnusError> {
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

    Ok(magnus::Ruby::get().expect("Ruby VM should be available").qnil().as_value())
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
            return Ok(true);
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
