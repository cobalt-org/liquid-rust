use std::fmt;

use num_traits;
use serde::ser::Impossible;
use serde::{self, Serialize};

use super::Object;
use super::Value;
use crate::error;

/// Convert a `T` into `liquid_value:Value`.
///
/// # Examples
///
/// ```rust
/// let s = "foo";
/// let value = liquid_value::to_value(s).unwrap();
/// assert_eq!(value, liquid_value::Value::scalar(s));
/// ```
pub fn to_value<T>(value: T) -> Result<Value, error::Error>
where
    T: Serialize,
{
    value.serialize(Serializer).map_err(|e| e.0)
}

#[derive(Debug)]
struct SerError(error::Error);

impl fmt::Display for SerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

impl ::std::error::Error for SerError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn source(&self) -> Option<&(dyn (::std::error::Error) + 'static)> {
        ::std::error::Error::source(&self.0)
    }
}

impl serde::ser::Error for SerError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerError(error::Error::with_msg(format!("{}", msg)))
    }
}

struct Serializer;

impl Serializer {
    #[inline]
    fn serialize_as_i32<T: num_traits::cast::NumCast>(self, value: T) -> Result<Value, SerError> {
        let value = num_traits::cast::cast::<T, i32>(value)
            .ok_or_else(|| SerError(error::Error::with_msg("Cannot fit number")))?;
        Ok(Value::scalar(value))
    }
}

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = SerError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value, SerError> {
        Ok(Value::scalar(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value, SerError> {
        Ok(Value::scalar(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value, SerError> {
        self.serialize_as_i32(value)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value, SerError> {
        self.serialize_f64(f64::from(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value, SerError> {
        Ok(Value::scalar(value))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value, SerError> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value, SerError> {
        Ok(Value::scalar(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value, SerError> {
        let vec = value.iter().map(|&b| Value::scalar(i32::from(b))).collect();
        Ok(Value::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value, SerError> {
        Ok(Value::Nil)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, SerError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, SerError> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, SerError>
    where
        T: Serialize,
    {
        value.serialize(Serializer)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, SerError>
    where
        T: Serialize,
    {
        let mut values = Object::new();
        values.insert(String::from(variant).into(), value.serialize(Serializer)?);
        Ok(Value::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value, SerError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value, SerError>
    where
        T: Serialize,
    {
        value.serialize(Serializer)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, SerError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, SerError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerError> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerError> {
        Ok(SerializeMap::Map {
            map: Object::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, SerError> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerError> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: Object::new(),
        })
    }
}

struct SerializeVec {
    vec: Vec<Value>,
}

struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

enum SerializeMap {
    Map {
        map: Object,
        next_key: Option<String>,
    },
}

struct SerializeStructVariant {
    name: String,
    map: Object,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerError> {
        Ok(Value::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, SerError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerError> {
        let mut object = Object::new();

        object.insert(self.name.into(), Value::Array(self.vec));

        Ok(Value::Object(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = SerError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut next_key, ..
            } => {
                *next_key = Some(r#try!(key.serialize(MapKeySerializer)));
                Ok(())
            }
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut map,
                ref mut next_key,
            } => {
                let key = next_key.take();
                // Panic because this indicates a bug in the program rather than an
                // expected failure.
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key.into(), value.serialize(Serializer)?);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Value, SerError> {
        match self {
            SerializeMap::Map { map, .. } => Ok(Value::Object(map)),
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> SerError {
    SerError(error::Error::with_msg("Key must be a string."))
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = SerError;

    type SerializeSeq = Impossible<String, SerError>;
    type SerializeTuple = Impossible<String, SerError>;
    type SerializeTupleStruct = Impossible<String, SerError>;
    type SerializeTupleVariant = Impossible<String, SerError>;
    type SerializeMap = Impossible<String, SerError>;
    type SerializeStruct = Impossible<String, SerError>;
    type SerializeStructVariant = Impossible<String, SerError>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map { .. } => {
                r#try!(serde::ser::SerializeMap::serialize_key(self, key));
                serde::ser::SerializeMap::serialize_value(self, value)
            }
        }
    }

    fn end(self) -> Result<Value, SerError> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
        }
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        self.map
            .insert(String::from(key).into(), value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, SerError> {
        let mut object = Object::new();

        object.insert(self.name.into(), Value::Object(self.map));

        Ok(Value::Object(object))
    }
}
