use std::borrow;

/// User-visible call trace
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Trace {
    trace: Option<borrow::Cow<'static, str>>,
    context: Vec<(borrow::Cow<'static, str>, borrow::Cow<'static, str>)>,
}

impl Trace {
    pub(crate) fn new(trace: borrow::Cow<'static, str>) -> Self {
        Self {
            trace: Some(trace),
            context: vec![],
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            trace: None,
            context: vec![],
        }
    }

    pub(crate) fn append_context(&mut self, key: borrow::Cow<'static, str>, value: borrow::Cow<'static, str>) {
        self.context.push((key, value));
    }

    pub(crate) fn get_trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| s.as_ref())
    }

    pub(crate) fn get_context(&self) -> &[(borrow::Cow<'static, str>, borrow::Cow<'static, str>)] {
        self.context.as_ref()
    }
}

