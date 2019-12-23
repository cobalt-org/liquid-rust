use std::fmt;

/// Queryable state for a `Value`.
///
/// See tables in https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum State {
    /// Is the value truthy?
    Truthy,
    /// Is the value the same as default initialized?
    DefaultValue,
    /// Is the value empty?
    Empty,
    /// Is the value blank?
    Blank,
}

impl State {
    /// A `Display` for a `Scalar` as source code.
    pub fn source(&self) -> StateSource {
        StateSource(*self)
    }

    /// A `Display` for a `Value` rendered for the user.
    pub fn render(&self) -> StateRendered {
        StateRendered(*self)
    }

    /// Interpret as a string.
    pub fn to_kstr(&self) -> kstring::KStringCow<'_> {
        kstring::KStringCow::default()
    }

    /// Query the value's state
    #[inline]
    pub fn query_state(&self, state: State) -> bool {
        match state {
            State::Truthy => self.is_truthy(),
            State::DefaultValue => self.is_default(),
            State::Empty => self.is_empty(),
            State::Blank => self.is_blank(),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            State::Truthy => false,
            State::DefaultValue => false,
            State::Empty => false,
            State::Blank => false,
        }
    }

    fn is_default(&self) -> bool {
        match self {
            State::Truthy => true,
            State::DefaultValue => true,
            State::Empty => true,
            State::Blank => true,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            State::Truthy => true,
            State::DefaultValue => true,
            State::Empty => true,
            State::Blank => true,
        }
    }

    fn is_blank(&self) -> bool {
        match self {
            State::Truthy => true,
            State::DefaultValue => true,
            State::Empty => true,
            State::Blank => true,
        }
    }

    /// Report the data type (generally for error reporting).
    pub fn type_name(&self) -> &'static str {
        match self {
            State::Truthy => "truthy",
            State::DefaultValue => "default",
            State::Empty => "empty",
            State::Blank => "blank",
        }
    }
}

/// A `Display` for a `State` as source code.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateSource(State);

impl fmt::Display for StateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.type_name())
    }
}

/// A `Display` for a `State` rendered for the user.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateRendered(State);

impl fmt::Display for StateRendered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
