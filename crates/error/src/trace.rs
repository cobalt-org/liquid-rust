/// User-visible call trace
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Trace {
    trace: Option<sstring::SString>,
    context: Vec<(sstring::SString, sstring::SString)>,
}

impl Trace {
    pub(crate) fn new(trace: sstring::SString) -> Self {
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

    pub(crate) fn append_context(&mut self, key: sstring::SString, value: sstring::SString) {
        self.context.push((key, value));
    }

    pub(crate) fn get_trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| s.as_str())
    }

    pub(crate) fn get_context(&self) -> &[(sstring::SString, sstring::SString)] {
        self.context.as_ref()
    }
}
