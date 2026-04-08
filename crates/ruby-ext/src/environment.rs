use magnus::{r_array::RArray, r_hash::RHash, Error as MagnusError, RModule, Value};

pub(crate) fn ext_env_default(ruby: &magnus::Ruby) -> Result<RHash, MagnusError> {
    let handle = ruby.hash_new();
    handle.aset("error_mode", "strict")?;
    handle.aset("tags", ruby.hash_new())?;
    handle.aset("filters", ruby.ary_new())?;
    handle.aset("file_system", ruby.qnil())?;
    Ok(handle)
}

pub(crate) fn ext_env_build(
    ruby: &magnus::Ruby,
    options: Option<RHash>,
) -> Result<RHash, MagnusError> {
    let handle = ext_env_default(ruby)?;
    if let Some(options) = options {
        handle.update(options)?;
    }
    Ok(handle)
}

pub(crate) fn ext_env_register_tag(
    _ruby: &magnus::Ruby,
    handle: RHash,
    name: String,
    klass: Value,
) -> Result<(), MagnusError> {
    let tags: RHash = handle.lookup("tags")?;
    tags.aset(name, klass)?;
    Ok(())
}

pub(crate) fn ext_env_register_filter(
    ruby: &magnus::Ruby,
    handle: RHash,
    filter_module: RModule,
) -> Result<(), MagnusError> {
    let filters: RArray = handle.lookup("filters")?;
    filters.push(filter_module)?;
    handle.aset("filters", filters)?;
    handle.aset("strainer_template", ruby.qnil())?;
    Ok(())
}
