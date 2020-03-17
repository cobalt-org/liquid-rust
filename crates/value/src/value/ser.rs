use kstring::KString;
use serde::{self, Serialize};

use crate::scalar::ser::ScalarSerializer;
use crate::ser::{SerError, SerializeMap, SerializeStructVariant, SerializeTupleVariant};
use crate::Object;
use crate::Value;

/// Convert a `T` into `liquid_value::Value`.
///
/// # Examples
///
/// ```rust
/// let s = "foo";
/// let value = liquid_value::to_value(&s).unwrap();
/// assert_eq!(value, liquid_value::Value::scalar(s));
/// ```
pub fn to_value<T>(value: &T) -> Result<Value, liquid_error::Error>
where
    T: Serialize,
{
    value.serialize(ValueSerializer).map_err(|e| e.into())
}

pub(crate) struct ValueSerializer;

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
        ScalarSerializer.serialize_bool(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value, SerError> {
        ScalarSerializer.serialize_i8(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value, SerError> {
        ScalarSerializer.serialize_i16(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value, SerError> {
        ScalarSerializer.serialize_i32(value).map(Value::Scalar)
    }

    fn serialize_i64(self, value: i64) -> Result<Value, SerError> {
        ScalarSerializer.serialize_i64(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value, SerError> {
        ScalarSerializer.serialize_u8(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value, SerError> {
        ScalarSerializer.serialize_u16(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value, SerError> {
        ScalarSerializer.serialize_u32(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value, SerError> {
        ScalarSerializer.serialize_u64(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value, SerError> {
        ScalarSerializer.serialize_f32(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value, SerError> {
        ScalarSerializer.serialize_f64(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value, SerError> {
        ScalarSerializer.serialize_char(value).map(Value::Scalar)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value, SerError> {
        ScalarSerializer.serialize_str(value).map(Value::Scalar)
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
            .map(Value::Scalar)
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

pub(crate) struct SerializeVec {
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
