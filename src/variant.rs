use failure::{bail, Error};
use log::debug;
use widestring::WideCStr;

use serde::de;
use serde::de::IntoDeserializer;

use winapi::shared::wtypes::*;
use winapi::um::oaidl::{VARIANT_n3, VARIANT};
use serde::Deserialize;
use std::fmt;

// See: https://msdn.microsoft.com/en-us/library/cc237864.aspx
const VARIANT_FALSE: i16 = 0x0000;

#[derive(Debug)]
pub enum Variant {
    Empty,
    Null,

    String(String),

    I2(i16),
    I4(i32),
    I8(i64),

    Bool(bool),


    UI1(u8),
    UI8(u64),
}

impl Variant {
    pub fn from_variant(vt: VARIANT) -> Result<Variant, Error> {
        let variant_type: VARTYPE = unsafe { vt.n1.n2().vt };

        println!("{:?}", variant_type);

        // See https://msdn.microsoft.com/en-us/library/cc237865.aspx for more info.
        // Rust can infer the return type of `vt.*Val()` calls,
        // but it's easier to read when the type is named explicitly.
        let variant_value = match variant_type as u32 {
            VT_BSTR => {
                let bstr_ptr: &BSTR = unsafe { vt.n1.n2().n3.bstrVal() };

                let prop_val: &WideCStr = unsafe { WideCStr::from_ptr_str(*bstr_ptr) };

                let property_value_as_string = prop_val.to_string()?;

                Variant::String(property_value_as_string)
            },
            VT_I2 => {
                let num: &i16 = unsafe { vt.n1.n2().n3.iVal() };

                Variant::I2(*num)
            },
            VT_I4 => {
                let num: &i32 = unsafe { vt.n1.n2().n3.lVal() };

                Variant::I4(*num)
            },
            VT_BOOL => {
                let value: &i16 = unsafe { vt.n1.n2().n3.boolVal() };

                match *value {
                    VARIANT_FALSE => Variant::Bool(false),
                    VARIANT_TRUE => Variant::Bool(true),
                    _ => bail!("Invalid bool value: {:#X}", value),
                }
            },
            VT_UI1 => {
                let num: &i8 = unsafe { vt.n1.n2().n3.cVal() };

                Variant::UI1(*num as u8)
            },
            VT_EMPTY => {
                Variant::Empty
            },
            VT_NULL => {
                Variant::Null
            },
            _ => bail!(
                "Converting from variant type {:#X} is not implemented yet",
                variant_type
            ),
        };

        debug!("Got {:?}", variant_value);

        Ok(variant_value)
    }
}


impl<'de> Deserialize<'de> for Variant {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Variant, D::Error>
        where
            D: serde::Deserializer<'de>,
    {

        struct VariantVisitor;

        impl<'de> de::Visitor<'de> for VariantVisitor {
            type Value = Variant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid variant value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Variant::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Variant::I8(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Variant::UI8(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                unimplemented!();
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Variant::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Variant::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Variant::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: de::SeqAccess<'de>,
            {
                unimplemented!();

                /*
                let mut vec = Vec::new();

                while let Some(elem) = try!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Value::Array(vec))
                */
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: de::MapAccess<'de>,
            {
                unimplemented!()
            }
        }

        deserializer.deserialize_any(VariantVisitor)
    }
}

