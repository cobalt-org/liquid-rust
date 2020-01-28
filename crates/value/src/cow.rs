use std::fmt;

use kstring::KStringCow;

use super::ArrayView;
use super::DisplayCow;
use super::ObjectView;
use super::ScalarCow;
use super::State;
use super::Value;
use super::ValueView;

/// Abstract the lifetime of a `Value`.
#[derive(Clone, Debug)]
pub enum ValueCow<'s> {
    /// A boxed `Value`
    Owned(Value),
    /// A borrowed `Value`
    Borrowed(&'s dyn ValueView),
}

impl<'s> ValueCow<'s> {
    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    pub fn into_owned(self) -> Value {
        match self {
            ValueCow::Owned(x) => x,
            ValueCow::Borrowed(x) => x.to_value(),
        }
    }

    /// Performs the conversion.
    pub fn as_view(&self) -> &dyn ValueView {
        match self {
            ValueCow::Owned(o) => o.as_view(),
            ValueCow::Borrowed(b) => *b,
        }
    }
}

impl<'s> ValueView for ValueCow<'s> {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        self.as_view().render()
    }
    fn source(&self) -> DisplayCow<'_> {
        self.as_view().source()
    }
    fn type_name(&self) -> &'static str {
        self.as_view().type_name()
    }
    fn query_state(&self, state: State) -> bool {
        self.as_view().query_state(state)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        self.as_view().to_kstr()
    }
    fn to_value(&self) -> Value {
        self.as_view().to_value()
    }

    fn as_scalar(&self) -> Option<ScalarCow<'_>> {
        self.as_view().as_scalar()
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        self.as_view().as_array()
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        self.as_view().as_object()
    }

    fn as_state(&self) -> Option<State> {
        self.as_view().as_state()
    }

    fn is_nil(&self) -> bool {
        self.as_view().is_nil()
    }
}

impl From<Value> for ValueCow<'static> {
    fn from(other: Value) -> Self {
        ValueCow::Owned(other)
    }
}

impl<'s> From<&'s Value> for ValueCow<'s> {
    fn from(other: &'s Value) -> Self {
        ValueCow::Borrowed(other.as_view())
    }
}
