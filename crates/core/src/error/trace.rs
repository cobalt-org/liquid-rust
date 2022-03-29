/// User-visible call trace
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Trace {
    trace: Option<crate::model::KString>,
    context: Vec<(crate::model::KString, crate::model::KString)>,
}

impl Trace {
    pub(crate) fn new(trace: crate::model::KString) -> Self {
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

    pub(crate) fn append_context(
        &mut self,
        key: crate::model::KString,
        value: crate::model::KString,
    ) {
        self.context.push((key, value));
    }

    pub(crate) fn get_trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| s.as_str())
    }

    pub(crate) fn get_context(&self) -> &[(crate::model::KString, crate::model::KString)] {
        self.context.as_ref()
    }
}
