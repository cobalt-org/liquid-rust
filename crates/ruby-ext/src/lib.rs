mod callbacks;
mod context;
mod environment;
mod errors;
mod template;
mod values;

use magnus::{function, Error, Module, Object, Ruby};

#[allow(clippy::needless_pass_by_value)]
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let liquid = ruby.define_module("Liquid")?;
    let ext = liquid.define_module("RustExtension")?;
    ext.define_class("NativeTemplate", ruby.class_object())?;

    ext.define_singleton_method("ext_parse", function!(template::ext_parse, 4))?;
    ext.define_singleton_method("ext_render", function!(template::ext_render, 2))?;
    ext.define_singleton_method("ext_render_strict", function!(template::ext_render_strict, 2))?;
    ext.define_singleton_method("ext_template_root", function!(template::ext_template_root, 1))?;
    ext.define_singleton_method("ext_template_errors", function!(template::ext_template_errors, 1))?;
    ext.define_singleton_method("ext_template_warnings", function!(template::ext_template_warnings, 1))?;
    ext.define_singleton_method("ext_debug_payload", function!(template::ext_debug_payload, 1))?;

    ext.define_singleton_method("ext_context_new", function!(context::ext_context_new, 3))?;
    ext.define_singleton_method("ext_context_set", function!(context::ext_context_set, 3))?;
    ext.define_singleton_method("ext_context_get", function!(context::ext_context_get, 2))?;
    ext.define_singleton_method("ext_context_has_key", function!(context::ext_context_has_key, 2))?;
    ext.define_singleton_method("ext_context_push", function!(context::ext_context_push, 2))?;
    ext.define_singleton_method("ext_context_pop", function!(context::ext_context_pop, 1))?;
    ext.define_singleton_method(
        "ext_context_find_variable",
        function!(context::ext_context_find_variable, 2),
    )?;

    ext.define_singleton_method("ext_env_default", function!(environment::ext_env_default, 0))?;
    ext.define_singleton_method("ext_env_build", function!(environment::ext_env_build, 1))?;
    ext.define_singleton_method("ext_env_register_tag", function!(environment::ext_env_register_tag, 3))?;
    ext.define_singleton_method(
        "ext_env_register_filter",
        function!(environment::ext_env_register_filter, 2),
    )?;

    Ok(())
}
