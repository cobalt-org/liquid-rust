use std::fmt;

use kstring::KString;
use num_traits;
use serde::ser::Impossible;
use serde::{self, Serialize};

use super::Object;
use super::Scalar;
use super::Value;

/// Convert a `T` into `liquid_value::Value`.
///
/// # Examples
///
/// ```rust
/// let s = "foo";
/// let value = liquid_value::to_value(s).unwrap();
/// assert_eq!(value, liquid_value::Value::scalar(s));
/// ```
pub fn to_value<T>(value: T) -> Result<Value, liquid_error::Error>
where
    T: Serialize,
{
    value.serialize(ValueSerializer).map_err(|e| e.0)
}

/// Convert a `T` into `liquid_value::Object`.
pub fn to_object<T>(value: T) -> Result<Object, liquid_error::Error>
where
    T: Serialize,
{
    value.serialize(ObjectSerializer).map_err(|e| e.0)
}

/// Convert a `T` into `liquid_value::Scalar`.
///
/// # Examples
///
/// ```rust
/// let s = "foo";
/// let value = liquid_value::to_scalar(s).unwrap();
/// assert_eq!(value, liquid_value::Scalar::new(s));
/// ```
pub fn to_scalar<T>(value: T) -> Result<Scalar, liquid_error::Error>
where
    T: Serialize,
{
    value.serialize(ScalarSerializer).map_err(|e| e.0)
}

#[derive(Debug)]
struct SerError(liquid_error::Error);

impl fmt::Display for SerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

impl ::std::error::Error for SerError {
    fn source(&self) -> Option<&(dyn (::std::error::Error) + 'static)> {
        ::std::error::Error::source(&self.0)
    }
}

impl serde::ser::Error for SerError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerError(liquid_error::Error::with_msg(format!("{}", msg)))
    }
}

struct ValueSerializer;

impl serde::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = SerError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant<Value>;
    type SerializeMap = SerializeMap<Value>;
    type SerializeStruct = SerializeMap<Value>;
    type SerializeStructVariant = SerializeStructVariant<Value>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_bool(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_i8(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_i16(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_i32(value)
            .map(|s| Value::Scalar(s))
    }

    fn serialize_i64(self, value: i64) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_i64(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_u8(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_u16(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_u32(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_u64(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_f32(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_f64(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_char(value)
            .map(|s| Value::Scalar(s))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_str(value)
            .map(|s| Value::Scalar(s))
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
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, SerError> {
        ScalarSerializer
            .serialize_unit_variant(name, variant_index, variant)
            .map(|s| Value::Scalar(s))
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
        value.serialize(ValueSerializer)
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
        values.insert(
            KString::from_static(variant),
            value.serialize(ValueSerializer)?,
        );
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
        value.serialize(ValueSerializer)
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
            name: KString::from_static(variant),
            vec: Vec::with_capacity(len),
            other: std::marker::PhantomData,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerError> {
        Ok(SerializeMap::Map {
            map: Object::new(),
            next_key: None,
            other: std::marker::PhantomData,
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
            name: KString::from_static(variant),
            map: Object::new(),
            other: std::marker::PhantomData,
        })
    }
}

struct SerializeVec {
    vec: Vec<Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        self.vec.push(value.serialize(ValueSerializer)?);
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

struct ObjectSerializer;

fn object_cannot_be_a_scalar() -> SerError {
    SerError(liquid_error::Error::with_msg("Object cannot be a scalar."))
}

fn object_cannot_be_an_array() -> SerError {
    SerError(liquid_error::Error::with_msg("Object cannot be an array."))
}

impl serde::Serializer for ObjectSerializer {
    type Ok = Object;
    type Error = SerError;

    type SerializeSeq = Impossible<Object, SerError>;
    type SerializeTuple = Impossible<Object, SerError>;
    type SerializeTupleStruct = Impossible<Object, SerError>;
    type SerializeTupleVariant = SerializeTupleVariant<Object>;
    type SerializeMap = SerializeMap<Object>;
    type SerializeStruct = SerializeMap<Object>;
    type SerializeStructVariant = SerializeStructVariant<Object>;

    #[inline]
    fn serialize_bool(self, _value: bool) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_i8(self, _value: i8) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_i16(self, _value: i16) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_i32(self, _value: i32) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    fn serialize_i64(self, _value: i64) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_u8(self, _value: u8) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_u16(self, _value: u16) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_u32(self, _value: u32) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_u64(self, _value: u64) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_f32(self, _value: f32) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_f64(self, _value: f64) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_char(self, _value: char) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_str(self, _value: &str) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_unit(self) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Object, SerError>
    where
        T: Serialize,
    {
        value.serialize(ObjectSerializer)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Object, SerError>
    where
        T: Serialize,
    {
        let mut values = Object::new();
        values.insert(
            KString::from_static(variant),
            value.serialize(ValueSerializer)?,
        );
        Ok(values)
    }

    #[inline]
    fn serialize_none(self) -> Result<Object, SerError> {
        Err(object_cannot_be_a_scalar())
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Object, SerError>
    where
        T: Serialize,
    {
        value.serialize(ObjectSerializer)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, SerError> {
        Err(object_cannot_be_an_array())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, SerError> {
        Err(object_cannot_be_an_array())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerError> {
        Err(object_cannot_be_an_array())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerError> {
        Ok(SerializeTupleVariant {
            name: KString::from_static(variant),
            vec: Vec::with_capacity(len),
            other: std::marker::PhantomData,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerError> {
        Ok(SerializeMap::Map {
            map: Object::new(),
            next_key: None,
            other: std::marker::PhantomData,
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
            name: KString::from_static(variant),
            map: Object::new(),
            other: std::marker::PhantomData,
        })
    }
}

struct SerializeTupleVariant<O> {
    name: KString,
    vec: Vec<Value>,
    other: std::marker::PhantomData<O>,
}

impl<O: From<Object>> serde::ser::SerializeTupleVariant for SerializeTupleVariant<O> {
    type Ok = O;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<O, SerError> {
        let mut object = Object::new();

        object.insert(self.name, Value::Array(self.vec));

        Ok(From::from(object))
    }
}

enum SerializeMap<O> {
    Map {
        map: Object,
        next_key: Option<KString>,
        other: std::marker::PhantomData<O>,
    },
}

impl<O: From<Object>> serde::ser::SerializeStruct for SerializeMap<O> {
    type Ok = O;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut next_key, ..
            } => {
                *next_key = Some(KString::from_static(key));
                serde::ser::SerializeMap::serialize_value(self, value)
            }
        }
    }

    fn end(self) -> Result<O, SerError> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
        }
    }
}

impl<O: From<Object>> serde::ser::SerializeMap for SerializeMap<O> {
    type Ok = O;
    type Error = SerError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut next_key, ..
            } => {
                *next_key = Some(key.serialize(MapKeySerializer)?);
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
                ..
            } => {
                let key = next_key.take();
                // Panic because this indicates a bug in the program rather than an
                // expected failure.
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key, value.serialize(ValueSerializer)?);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<O, SerError> {
        match self {
            SerializeMap::Map { map, .. } => Ok(From::from(map)),
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> SerError {
    SerError(liquid_error::Error::with_msg("Key must be a string."))
}

impl serde::Serializer for MapKeySerializer {
    type Ok = KString;
    type Error = SerError;

    type SerializeSeq = Impossible<KString, SerError>;
    type SerializeTuple = Impossible<KString, SerError>;
    type SerializeTupleStruct = Impossible<KString, SerError>;
    type SerializeTupleVariant = Impossible<KString, SerError>;
    type SerializeMap = Impossible<KString, SerError>;
    type SerializeStruct = Impossible<KString, SerError>;
    type SerializeStructVariant = Impossible<KString, SerError>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(KString::from_static(variant))
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
        Ok(value.to_string().into())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string().into())
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
            s.into()
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(KString::from_ref(value))
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

struct SerializeStructVariant<O> {
    name: KString,
    map: Object,
    other: std::marker::PhantomData<O>,
}

impl<O: From<Object>> serde::ser::SerializeStructVariant for SerializeStructVariant<O> {
    type Ok = O;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), SerError>
    where
        T: Serialize,
    {
        self.map
            .insert(KString::from_static(key), value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<O, SerError> {
        let mut object = Object::new();

        object.insert(self.name, Value::Object(self.map));

        Ok(From::from(object))
    }
}

struct ScalarSerializer;

fn scalar_must_be_a_string() -> SerError {
    SerError(liquid_error::Error::with_msg("Scalar must be a string."))
}

impl serde::Serializer for ScalarSerializer {
    type Ok = Scalar;
    type Error = SerError;

    type SerializeSeq = Impossible<Scalar, SerError>;
    type SerializeTuple = Impossible<Scalar, SerError>;
    type SerializeTupleStruct = Impossible<Scalar, SerError>;
    type SerializeTupleVariant = Impossible<Scalar, SerError>;
    type SerializeMap = Impossible<Scalar, SerError>;
    type SerializeStruct = Impossible<Scalar, SerError>;
    type SerializeStructVariant = Impossible<Scalar, SerError>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Scalar, SerError> {
        Ok(Scalar::new(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Scalar, SerError> {
        Ok(Scalar::new(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Scalar, SerError> {
        serialize_as_i32(value)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Scalar, SerError> {
        self.serialize_f64(f64::from(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Scalar, SerError> {
        Ok(Scalar::new(value))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Scalar, SerError> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Scalar, SerError> {
        Ok(Scalar::new(KString::from_ref(value)))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Scalar, SerError> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Scalar, SerError>
    where
        T: Serialize,
    {
        value.serialize(ScalarSerializer)
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
        Err(scalar_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(scalar_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(scalar_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(scalar_must_be_a_string())
    }
}

#[inline]
fn serialize_as_i32<T: num_traits::cast::NumCast>(value: T) -> Result<Scalar, SerError> {
    let value = num_traits::cast::cast::<T, i32>(value)
        .ok_or_else(|| SerError(liquid_error::Error::with_msg("Cannot fit number")))?;
    Ok(Scalar::new(value))
}
