use std::{any::type_name, collections::HashMap, fmt::Display};

use crate::Variant;
use serde::{
    ser::{Impossible, SerializeStruct},
    Serialize, Serializer,
};
use thiserror::Error;

pub struct VariantStructSerializer {
    variant_map: HashMap<String, Variant>,
}

#[derive(Debug, Error)]
pub enum VariantStructSerializerError {
    #[error("Unknown error when serializing struct:\n{0}")]
    Unknown(String),
    #[error("VariantSerializer can only be used to serialize structs.")]
    ExpectedStruct,
    #[error("{0} cannot be serialized to a Variant.")]
    UnsupportedVariantType(String),
}

impl VariantStructSerializer {
    pub fn new() -> Self {
        Self {
            variant_map: HashMap::new(),
        }
    }
}

impl serde::ser::Error for VariantStructSerializerError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        VariantStructSerializerError::Unknown(msg.to_string())
    }
}

macro_rules! serialize_type_stub {
    ($signature:ident, $type:ty) => {
        fn $signature(self, _v: $type) -> Result<Self::Ok, Self::Error> {
            Err(VariantStructSerializerError::ExpectedStruct)
        }
    };
}

impl Serializer for VariantStructSerializer {
    type Ok = HashMap<String, Variant>;

    type Error = VariantStructSerializerError;

    type SerializeStruct = Self;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(HashMap::new())
    }

    // The following is code for a generic Serializer implementation not relevant to this use case

    type SerializeSeq = Impossible<Self::Ok, VariantStructSerializerError>;
    type SerializeTuple = Impossible<Self::Ok, VariantStructSerializerError>;
    type SerializeTupleStruct = Impossible<Self::Ok, VariantStructSerializerError>;
    type SerializeTupleVariant = Impossible<Self::Ok, VariantStructSerializerError>;
    type SerializeMap = Impossible<Self::Ok, VariantStructSerializerError>;
    type SerializeStructVariant = Impossible<Self::Ok, VariantStructSerializerError>;

    serialize_type_stub!(serialize_bool, bool);
    serialize_type_stub!(serialize_i8, i8);
    serialize_type_stub!(serialize_i16, i16);
    serialize_type_stub!(serialize_i32, i32);
    serialize_type_stub!(serialize_i64, i64);
    serialize_type_stub!(serialize_u8, u8);
    serialize_type_stub!(serialize_u16, u16);
    serialize_type_stub!(serialize_u32, u32);
    serialize_type_stub!(serialize_u64, u64);
    serialize_type_stub!(serialize_f32, f32);
    serialize_type_stub!(serialize_f64, f64);
    serialize_type_stub!(serialize_char, char);
    serialize_type_stub!(serialize_str, &str);
    serialize_type_stub!(serialize_bytes, &[u8]);

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(VariantStructSerializerError::ExpectedStruct)
    }
}

macro_rules! match_type_unsafe {
    ($target_type:ty, $value:ident) => {
        || {
            if typeid::of::<T>() == typeid::of::<$target_type>() {
                Some(Variant::from(*unsafe {
                    std::mem::transmute_copy::<&T, &$target_type>(&$value)
                }))
            } else {
                None
            }
        }
    };
}

impl SerializeStruct for VariantStructSerializer {
    type Ok = <VariantStructSerializer as Serializer>::Ok;

    type Error = VariantStructSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let variant = match_type_unsafe!((), value)()
            .or_else(match_type_unsafe!(i8, value))
            .or_else(match_type_unsafe!(i16, value))
            .or_else(match_type_unsafe!(i32, value))
            .or_else(match_type_unsafe!(i64, value))
            .or_else(match_type_unsafe!(f32, value))
            .or_else(match_type_unsafe!(f64, value))
            .or_else(match_type_unsafe!(bool, value))
            .or_else(match_type_unsafe!(u8, value))
            .or_else(match_type_unsafe!(u16, value))
            .or_else(match_type_unsafe!(u32, value))
            .or_else(match_type_unsafe!(u64, value))
            .or_else(|| {
                if typeid::of::<T>() == typeid::of::<String>() {
                    Some(Variant::from(
                        unsafe { std::mem::transmute_copy::<&T, &String>(&value) }.clone(),
                    ))
                } else {
                    None
                }
            });

        match variant {
            Some(value) => {
                self.variant_map.insert(key.to_string(), value);
                Ok(())
            }
            None => Err(VariantStructSerializerError::UnsupportedVariantType(
                type_name::<T>().to_string(),
            )),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.variant_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestStruct {
        empty: (),

        string: String,

        i1: i8,
        i2: i16,
        i4: i32,
        i8: i64,

        r4: f32,
        r8: f64,

        bool: bool,

        ui1: u8,
        ui2: u16,
        ui4: u32,
        ui8: u64,
    }

    #[test]
    fn it_serialize_struct() {
        let test_struct = TestStruct {
            empty: (),
            string: "Test String".to_string(),

            i1: i8::MAX,
            i2: i16::MAX,
            i4: i32::MAX,
            i8: i64::MAX,

            r4: f32::MAX,
            r8: f64::MAX,

            bool: false,

            ui1: u8::MAX,
            ui2: u16::MAX,
            ui4: u32::MAX,
            ui8: u64::MAX,
        };
        let expected_field_map: HashMap<String, Variant> = [
            ("empty".to_string(), Variant::Empty),
            (
                "string".to_string(),
                Variant::String("Test String".to_string()),
            ),
            ("i1".to_string(), Variant::I1(i8::MAX)),
            ("i2".to_string(), Variant::I2(i16::MAX)),
            ("i4".to_string(), Variant::I4(i32::MAX)),
            ("i8".to_string(), Variant::I8(i64::MAX)),
            ("r4".to_string(), Variant::R4(f32::MAX)),
            ("r8".to_string(), Variant::R8(f64::MAX)),
            ("bool".to_string(), Variant::Bool(false)),
            ("ui1".to_string(), Variant::UI1(u8::MAX)),
            ("ui2".to_string(), Variant::UI2(u16::MAX)),
            ("ui4".to_string(), Variant::UI4(u32::MAX)),
            ("ui8".to_string(), Variant::UI8(u64::MAX)),
        ]
        .into_iter()
        .collect();

        let variant_serializer = VariantStructSerializer::new();
        let field_map = test_struct.serialize(variant_serializer).unwrap();

        assert_eq!(field_map, expected_field_map);
    }
}
