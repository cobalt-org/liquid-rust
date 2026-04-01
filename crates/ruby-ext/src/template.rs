use liquid::ParserBuilder;
use liquid::model::{DisplayCow, KStringCow, State, Value as LiquidValue};
use liquid::ObjectView;
use liquid::Template as LiquidTemplate;
use liquid::ValueView;
use magnus::{
    r_array::RArray, r_hash::RHash, value::ReprValue, Error as MagnusError, TryConvert, Value,
};

use crate::errors;
use crate::values;

pub(crate) fn ext_parse(
    ruby: &magnus::Ruby,
    source: String,
    line_numbers: bool,
    error_mode: Option<String>,
    environment: Option<RHash>,
) -> Result<RHash, MagnusError> {
    let template = parse_template(ruby, &source)?;
    let handle = ruby.hash_new();
    handle.aset("source", source)?;
    handle.aset("line_numbers", line_numbers)?;
    handle.aset("error_mode", error_mode.unwrap_or_else(|| "strict".to_string()))?;
    handle.aset(
        "environment",
        environment.map_or_else(|| ruby.qnil().as_value(), ReprValue::as_value),
    )?;
    handle.aset("errors", ruby.ary_new())?;
    handle.aset("warnings", ruby.ary_new())?;
    handle.aset("root", build_root_handle(ruby, &template)?)?;
    Ok(handle)
}

pub(crate) fn ext_render(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
) -> Result<String, MagnusError> {
    render_internal(ruby, handle, context_or_assigns, false)
}

pub(crate) fn ext_render_strict(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
) -> Result<String, MagnusError> {
    render_internal(ruby, handle, context_or_assigns, true)
}

pub(crate) fn ext_template_root(
    _ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<Value, MagnusError> {
    handle.lookup("root")
}

pub(crate) fn ext_template_errors(
    _ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<Value, MagnusError> {
    handle.lookup("errors")
}

pub(crate) fn ext_template_warnings(
    _ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<Value, MagnusError> {
    handle.lookup("warnings")
}

pub(crate) fn ext_debug_payload(
    _ruby: &magnus::Ruby,
    context_or_assigns: Value,
) -> Result<Vec<String>, MagnusError> {
    let globals = globals_from_context(context_or_assigns)?;
    Ok(globals.keys().map(|key| key.to_string()).collect())
}

fn render_internal(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
    strict: bool,
) -> Result<String, MagnusError> {
    let source: String = handle.lookup("source")?;
    let template = parse_template(ruby, &source)?;
    let globals = globals_from_context(context_or_assigns)?;
    let strict_variables = strict_variables_enabled(context_or_assigns);
    let lenient_globals = LenientObject::new(&globals as &dyn ObjectView);
    let globals: &dyn ObjectView = if strict_variables {
        &globals
    } else {
        &lenient_globals
    };

    match template.render(globals) {
        Ok(rendered) => Ok(rendered),
        Err(error) => {
            let message = error.to_string();
            let errors: RArray = handle.lookup("errors")?;
            errors.push(message.clone())?;
            if strict {
                Err(errors::runtime_error(ruby, message))
            } else {
                Ok(String::new())
            }
        }
    }
}

fn parse_template(ruby: &magnus::Ruby, source: &str) -> Result<LiquidTemplate, MagnusError> {
    let parser = ParserBuilder::with_stdlib()
        .build()
        .map_err(|error| errors::runtime_error(ruby, error.to_string()))?;

    parser
        .parse(source)
        .map_err(|error| errors::syntax_error(ruby, error.to_string()))
}

fn build_root_handle(ruby: &magnus::Ruby, _template: &LiquidTemplate) -> Result<RArray, MagnusError> {
    let nodes = ruby.ary_new();
    Ok(nodes)
}

fn globals_from_context(context_or_assigns: Value) -> Result<values::RenderObject, MagnusError> {
    if let Ok(payload) = String::try_convert(context_or_assigns) {
        return values::json_to_object(&payload).map(values::liquid_to_render_object);
    }

    if let Some(handle) = RHash::from_value(context_or_assigns) {
        if let Some(scopes) = handle.get("scopes").and_then(RArray::from_value) {
            let mut merged = values::RenderObject::new();
            for idx in 0..scopes.len() {
                let scope: RHash = scopes.entry(idx as isize)?;
                let object = values::ruby_to_render_object(scope.as_value())?;
                for (key, value) in object {
                    merged.insert(key, value);
                }
            }
            return Ok(merged);
        }

        return values::ruby_to_render_object(handle.as_value());
    }

    values::ruby_to_render_object(context_or_assigns)
}

fn strict_variables_enabled(context_or_assigns: Value) -> bool {
    RHash::from_value(context_or_assigns)
        .and_then(|handle| handle.get("strict_variables"))
        .and_then(|value| bool::try_convert(value).ok())
        .unwrap_or(false)
}

static NIL_VALUE: LiquidValue = LiquidValue::Nil;

struct LenientObject<'a> {
    inner: &'a dyn ObjectView,
}

impl<'a> LenientObject<'a> {
    fn new(inner: &'a dyn ObjectView) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for LenientObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl ValueView for LenientObject<'_> {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self.inner.as_debug()
    }

    fn render(&self) -> DisplayCow<'_> {
        self.inner.render()
    }

    fn source(&self) -> DisplayCow<'_> {
        self.inner.source()
    }

    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }

    fn query_state(&self, state: State) -> bool {
        self.inner.query_state(state)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        self.inner.to_kstr()
    }

    fn to_value(&self) -> LiquidValue {
        self.inner.to_value()
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for LenientObject<'_> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.inner.size()
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        Box::new(
            self.inner
                .keys()
                .map(|key| KStringCow::from_string(key.into_owned().to_string())),
        )
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        self.inner.values()
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        Box::new(
            self.inner
                .iter()
                .map(|(key, value)| (KStringCow::from_string(key.into_owned().to_string()), value)),
        )
    }

    fn contains_key(&self, _index: &str) -> bool {
        true
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        self.inner
            .get(index)
            .map(|value| value as &dyn ValueView)
            .or(Some(&NIL_VALUE as &dyn ValueView))
    }
}
