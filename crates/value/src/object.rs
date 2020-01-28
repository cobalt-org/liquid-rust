use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

use kstring::KStringCow;
use liquid_error::{Error, Result};

use crate::map;
use crate::DisplayCow;
use crate::PathRef;
use crate::State;
use crate::{Value, ValueView};

/// Accessor for objects.
pub trait ObjectView: ValueView {
    /// Cast to ValueView
    fn as_value(&self) -> &dyn ValueView;

    /// Returns the number of elements.
    fn size(&self) -> i32;

    /// Keys available for lookup.
    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k>;
    /// Keys available for lookup.
    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k>;
    /// Returns an iterator .
    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k>;

    /// Access a contained `BoxedValue`.
    fn contains_key(&self, index: &str) -> bool;
    /// Access a contained `Value`.
    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView>;

    /// Find a `ValueView` nested in an `ObjectView`
    #[inline]
    fn try_find<'o>(&'o self, path: PathRef<'_, '_>) -> Option<&'o dyn ValueView> {
        let mut indexes = path.iter();
        let key = indexes.next()?;
        let key = key.to_kstr();
        let value = self.get(key.as_str())?;

        indexes.fold(Some(value), |value, index| {
            let value = value?;
            if let Some(arr) = value.as_array() {
                if let Some(index) = index.to_integer() {
                    arr.get(index)
                } else {
                    match &*index.to_kstr() {
                        "first" => arr.first(),
                        "last" => arr.last(),
                        _ => None,
                    }
                }
            } else if let Some(obj) = value.as_object() {
                obj.get(index.to_kstr().as_str())
            } else {
                None
            }
        })
    }

    /// Find a `ValueView` nested in an `ObjectView`
    #[inline]
    fn find<'o>(&'o self, path: PathRef<'_, '_>) -> Result<&'o dyn ValueView> {
        if let Some(res) = self.try_find(path) {
            Ok(res)
        } else {
            for cur_idx in 1..path.len() {
                let subpath_end = path.len() - cur_idx;
                let subpath = &path[0..subpath_end];
                if let Some(parent) = self.try_find(subpath) {
                    let subpath = itertools::join(subpath.iter().map(ValueView::render), ".");
                    let requested = &path[subpath_end];
                    let available = if let Some(arr) = parent.as_array() {
                        let mut available = vec![
                            KStringCow::from_static("first"),
                            KStringCow::from_static("last"),
                        ];
                        if 0 < arr.size() {
                            available.insert(
                                0,
                                KStringCow::from_string(format!("0..{}", arr.size() - 1)),
                            );
                        }
                        available
                    } else if let Some(obj) = parent.as_object() {
                        let available: Vec<_> = obj.keys().collect();
                        available
                    } else {
                        Vec::new()
                    };
                    let available = itertools::join(available.iter(), ", ");
                    return Error::with_msg("Unknown index")
                        .context("variable", subpath)
                        .context("requested index", format!("{}", requested.render()))
                        .context("available indexes", available)
                        .into_err();
                }
            }

            let requested = path
                .get(0)
                .expect("`Path` guarantees at least one element")
                .to_kstr()
                .into_owned();
            let available = itertools::join(self.keys(), ", ");
            Error::with_msg("Unknown variable")
                .context("requested variable", requested)
                .context("available variables", available)
                .into_err()
        }
    }
}

/// Type representing a Liquid object, payload of the `Value::Object` variant
pub type Object = map::Map;

impl ValueView for Object {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ObjectRender { s: self }))
    }
    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ObjectSource { s: self }))
    }
    fn type_name(&self) -> &'static str {
        "object"
    }
    fn query_state(&self, state: State) -> bool {
        match state {
            State::Truthy => true,
            State::DefaultValue | State::Empty | State::Blank => self.is_empty(),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        let s = ObjectRender { s: self }.to_string();
        KStringCow::from_string(s)
    }
    fn to_value(&self) -> Value {
        Value::Object(self.clone())
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for Object {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i32 {
        self.len() as i32
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        let keys = Object::keys(self).map(|s| s.as_ref().into());
        Box::new(keys)
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let i = Object::values(self).map(|v| v.as_view());
        Box::new(i)
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        let i = Object::iter(self).map(|(k, v)| (k.as_str().into(), v.as_view()));
        Box::new(i)
    }

    fn contains_key(&self, index: &str) -> bool {
        Object::contains_key(self, index)
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        Object::get(self, index).map(|v| v.as_view())
    }
}

/// Owned object index
pub trait ObjectIndex:
    fmt::Debug + fmt::Display + Ord + std::hash::Hash + Eq + std::borrow::Borrow<str>
{
    /// Borrow the index
    fn as_str(&self) -> &str;
}

impl ObjectIndex for String {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl ObjectIndex for kstring::KString {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl<K: ObjectIndex, V: ValueView, S: ::std::hash::BuildHasher> ValueView for HashMap<K, V, S> {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ObjectRender { s: self }))
    }
    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ObjectSource { s: self }))
    }
    fn type_name(&self) -> &'static str {
        "object"
    }
    fn query_state(&self, state: State) -> bool {
        match state {
            State::Truthy => true,
            State::DefaultValue | State::Empty | State::Blank => self.is_empty(),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        let s = ObjectRender { s: self }.to_string();
        KStringCow::from_string(s)
    }
    fn to_value(&self) -> Value {
        Value::Object(
            self.iter()
                .map(|(k, v)| (kstring::KString::from_ref(k.as_str()), v.to_value()))
                .collect(),
        )
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl<K: ObjectIndex, V: ValueView, S: ::std::hash::BuildHasher> ObjectView for HashMap<K, V, S> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i32 {
        self.len() as i32
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        let keys = HashMap::keys(self).map(|s| s.as_str().into());
        Box::new(keys)
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let i = HashMap::values(self).map(as_view);
        Box::new(i)
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        let i = HashMap::iter(self).map(|(k, v)| (k.as_str().into(), as_view(v)));
        Box::new(i)
    }

    fn contains_key(&self, index: &str) -> bool {
        HashMap::contains_key(self, index)
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        HashMap::get(self, index).map(as_view)
    }
}

impl<K: ObjectIndex, V: ValueView> ValueView for BTreeMap<K, V> {
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ObjectRender { s: self }))
    }
    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(ObjectSource { s: self }))
    }
    fn type_name(&self) -> &'static str {
        "object"
    }
    fn query_state(&self, state: State) -> bool {
        match state {
            State::Truthy => true,
            State::DefaultValue | State::Empty | State::Blank => self.is_empty(),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        let s = ObjectRender { s: self }.to_string();
        KStringCow::from_string(s)
    }
    fn to_value(&self) -> Value {
        Value::Object(
            self.iter()
                .map(|(k, v)| (kstring::KString::from_ref(k.as_str()), v.to_value()))
                .collect(),
        )
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl<K: ObjectIndex, V: ValueView> ObjectView for BTreeMap<K, V> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i32 {
        self.len() as i32
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        let keys = BTreeMap::keys(self).map(|s| s.as_str().into());
        Box::new(keys)
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let i = BTreeMap::values(self).map(as_view);
        Box::new(i)
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        let i = BTreeMap::iter(self).map(|(k, v)| (k.as_str().into(), as_view(v)));
        Box::new(i)
    }

    fn contains_key(&self, index: &str) -> bool {
        BTreeMap::contains_key(self, index)
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        BTreeMap::get(self, index).map(as_view)
    }
}

fn as_view<T: ValueView>(value: &T) -> &dyn ValueView {
    value
}

#[derive(Debug)]
#[doc(hidden)]
pub struct ObjectSource<'s, O: ObjectView> {
    s: &'s O,
}

impl<'s, O: ObjectView> ObjectSource<'s, O> {
    #[doc(hidden)]
    pub fn new(other: &'s O) -> Self {
        Self { s: other }
    }
}

impl<'s, O: ObjectView> fmt::Display for ObjectSource<'s, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (k, v) in self.s.iter() {
            write!(f, r#""{}": {}, "#, k, v.render())?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub struct ObjectRender<'s, O: ObjectView> {
    s: &'s O,
}

impl<'s, O: ObjectView> ObjectRender<'s, O> {
    #[doc(hidden)]
    pub fn new(other: &'s O) -> Self {
        Self { s: other }
    }
}

impl<'s, O: ObjectView> fmt::Display for ObjectRender<'s, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in self.s.iter() {
            write!(f, "{}{}", k, v.render())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_object() {
        let obj = Object::new();
        println!("{}", obj.source());
        let object: &dyn ObjectView = &obj;
        println!("{}", object.source());
        let view: &dyn ValueView = object.as_value();
        println!("{}", view.source());
    }
}
