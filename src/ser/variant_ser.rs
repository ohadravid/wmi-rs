//! This module implements a custom serializer type, [`VariantStructSerializer`],
//! to serialize a Rust struct into a HashMap mapping field name strings to [`Variant`] values
use std::{any::type_name, collections::HashMap, fmt::Display};

use crate::Variant;
use serde::{
    ser::{Impossible, SerializeStruct},
    Serialize, Serializer,
};
use thiserror::Error;

macro_rules! serialize_struct_err_stub {
    ($signature:ident, $type:ty) => {
        fn $signature(self, _v: $type) -> Result<Self::Ok, Self::Error> {
            Err(VariantSerializerError::ExpectedStruct)
        }
    };
}

macro_rules! serialize_variant_err_stub {
    ($signature:ident, $type:ty) => {
        fn $signature(self, _v: $type) -> Result<Self::Ok, Self::Error> {
            Err(VariantSerializerError::UnsupportedVariantType(
                type_name::<$type>().to_string(),
            ))
        }
    };
}

macro_rules! serialize_variant {
    ($signature:ident, $type:ty) => {
        fn $signature(self, v: $type) -> Result<Self::Ok, Self::Error> {
            Ok(Variant::from(v))
        }
    };
}

struct VariantSerializer {}

impl Serializer for VariantSerializer {
    type Ok = Variant;
    type Error = VariantSerializerError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    serialize_variant!(serialize_bool, bool);
    serialize_variant!(serialize_i8, i8);
    serialize_variant!(serialize_i16, i16);
    serialize_variant!(serialize_i32, i32);
    serialize_variant!(serialize_i64, i64);
    serialize_variant!(serialize_u8, u8);
    serialize_variant!(serialize_u16, u16);
    serialize_variant!(serialize_u32, u32);
    serialize_variant!(serialize_u64, u64);
    serialize_variant!(serialize_f32, f32);
    serialize_variant!(serialize_f64, f64);

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Variant::Empty)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Variant::from(v.to_string()))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Variant::from(variant.to_string()))
    }

    // Generic serializer code not relevant to this use case

    serialize_variant_err_stub!(serialize_char, char);
    serialize_variant_err_stub!(serialize_bytes, &[u8]);

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "None".to_string(),
        ))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantSerializerError::UnsupportedVariantType(
            type_name::<T>().to_string(),
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "Sequence".to_string(),
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "Tuple".to_string(),
        ))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            name.to_string(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(format!(
            "{variant}::{name}"
        )))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "Map".to_string(),
        ))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            name.to_string(),
        ))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(format!(
            "{variant}::{name}"
        )))
    }
}

#[derive(Default)]
pub struct VariantStructSerializer {
    variant_map: HashMap<String, Variant>,
}

#[derive(Debug, Error)]
pub enum VariantSerializerError {
    #[error("Unknown error when serializing struct:\n{0}")]
    Unknown(String),
    #[error("VariantStructSerializer can only be used to serialize structs.")]
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

impl serde::ser::Error for VariantSerializerError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        VariantSerializerError::Unknown(msg.to_string())
    }
}

impl Serializer for VariantStructSerializer {
    type Ok = HashMap<String, Variant>;

    type Error = VariantSerializerError;

    type SerializeStruct = Self;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(HashMap::new())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // The following is code for a generic Serializer implementation not relevant to this use case

    type SerializeSeq = Impossible<Self::Ok, VariantSerializerError>;
    type SerializeTuple = Impossible<Self::Ok, VariantSerializerError>;
    type SerializeTupleStruct = Impossible<Self::Ok, VariantSerializerError>;
    type SerializeTupleVariant = Impossible<Self::Ok, VariantSerializerError>;
    type SerializeMap = Impossible<Self::Ok, VariantSerializerError>;
    type SerializeStructVariant = Impossible<Self::Ok, VariantSerializerError>;

    serialize_struct_err_stub!(serialize_bool, bool);
    serialize_struct_err_stub!(serialize_i8, i8);
    serialize_struct_err_stub!(serialize_i16, i16);
    serialize_struct_err_stub!(serialize_i32, i32);
    serialize_struct_err_stub!(serialize_i64, i64);
    serialize_struct_err_stub!(serialize_u8, u8);
    serialize_struct_err_stub!(serialize_u16, u16);
    serialize_struct_err_stub!(serialize_u32, u32);
    serialize_struct_err_stub!(serialize_u64, u64);
    serialize_struct_err_stub!(serialize_f32, f32);
    serialize_struct_err_stub!(serialize_f64, f64);
    serialize_struct_err_stub!(serialize_char, char);
    serialize_struct_err_stub!(serialize_str, &str);
    serialize_struct_err_stub!(serialize_bytes, &[u8]);

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
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
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(VariantSerializerError::ExpectedStruct)
    }
}

impl SerializeStruct for VariantStructSerializer {
    type Ok = <VariantStructSerializer as Serializer>::Ok;

    type Error = VariantSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let variant = value.serialize(VariantSerializer {});
        match variant {
            Ok(value) => {
                self.variant_map.insert(key.to_string(), value);
                Ok(())
            }
            Err(_) => Err(VariantSerializerError::UnsupportedVariantType(
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

    #[derive(Serialize)]
    struct NewtypeTest(u32);
    #[derive(Serialize)]
    struct NewtypeTestWrapper {
        newtype: NewtypeTest,
    }
    #[test]
    fn it_serialize_newtype() {
        let test_struct = NewtypeTestWrapper {
            newtype: NewtypeTest(17),
        };

        let expected_field_map: HashMap<String, Variant> =
            [("newtype".to_string(), Variant::UI4(17))]
                .into_iter()
                .collect();

        let field_map = test_struct
            .serialize(VariantStructSerializer::new())
            .unwrap();

        assert_eq!(field_map, expected_field_map);
    }

    #[derive(Serialize)]
    struct UnitTest;

    #[test]
    fn it_serialize_unit() {
        let expected_field_map = HashMap::new();
        let field_map = UnitTest {}
            .serialize(VariantStructSerializer::new())
            .unwrap();

        assert_eq!(field_map, expected_field_map);
    }

    #[derive(Serialize)]
    #[allow(dead_code)]
    enum EnumTest {
        NTFS,
        FAT32,
        ReFS,
    }

    #[derive(Serialize)]
    struct EnumStructTest {
        enum_test: EnumTest,
    }

    #[test]
    fn it_serialize_enum() {
        let test_enum_struct = EnumStructTest {
            enum_test: EnumTest::NTFS,
        };

        let expected_field_map = [("enum_test".to_string(), Variant::from("NTFS".to_string()))]
            .into_iter()
            .collect();

        let field_map = test_enum_struct
            .serialize(VariantStructSerializer::new())
            .unwrap();

        assert_eq!(field_map, expected_field_map);
    }
}
