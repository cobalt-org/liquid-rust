use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum EnumIndex {
    Key(String),
    Index(isize),
}

/// An index into a `liquid_value::Value`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Index {
    part: EnumIndex,
}

impl Index {
    /// Create an `Object` `Index`.
    pub fn with_key<S: Into<String>>(key: S) -> Self {
        let part = EnumIndex::Key(key.into());
        Self { part }
    }

    /// Create an `Array` `Index`.
    pub fn with_index(index: isize) -> Self {
        let part = EnumIndex::Index(index);
        Self { part }
    }

    /// Check if `Object` `Index`.
    pub fn is_key(&self) -> bool {
        match self.part {
            EnumIndex::Key(_) => true,
            EnumIndex::Index(_) => false,
        }
    }

    /// Check if `Array` `Index`.
    pub fn is_index(&self) -> bool {
        match self.part {
            EnumIndex::Key(_) => false,
            EnumIndex::Index(_) => true,
        }
    }

    /// Return the `Object` `Index`.
    pub fn as_key(&self) -> Option<&str> {
        match self.part {
            EnumIndex::Key(ref k) => Some(k),
            EnumIndex::Index(_) => None,
        }
    }

    /// Return the `Array` `Index`.
    pub fn as_index(&self) -> Option<isize> {
        match self.part {
            EnumIndex::Key(_) => None,
            EnumIndex::Index(k) => Some(k),
        }
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.part {
            EnumIndex::Key(ref k) => write!(f, "{}", k),
            EnumIndex::Index(k) => write!(f, "{}", k),
        }
    }
}

impl From<isize> for Index {
    fn from(k: isize) -> Self {
        Self::with_index(k)
    }
}

impl From<String> for Index {
    fn from(k: String) -> Self {
        Self::with_key(k)
    }
}

impl<'a> From<&'a str> for Index {
    fn from(k: &'a str) -> Self {
        Self::with_key(k)
    }
}
