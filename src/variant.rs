use failure::{bail, Error};
use log::debug;
use widestring::WideCStr;

use serde::de;
use serde::forward_to_deserialize_any;

use crate::safearray::get_string_array;
use serde::de::IntoDeserializer;
use serde::Deserialize;
use std::fmt;
use winapi::shared::wtypes::*;
use winapi::um::oaidl::SAFEARRAY;
use winapi::um::oaidl::{VARIANT_n3, VARIANT};

// See: https://msdn.microsoft.com/en-us/library/cc237864.aspx
const VARIANT_FALSE: i16 = 0x0000;

#[derive(Debug, PartialEq, Hash)]
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

    Array(Vec<Variant>),
}

impl Variant {
    pub fn from_variant(vt: VARIANT) -> Result<Variant, Error> {
        let variant_type: VARTYPE = unsafe { vt.n1.n2().vt };

        if variant_type as u32 & VT_ARRAY == VT_ARRAY {
            let array: &*mut SAFEARRAY = unsafe { vt.n1.n2().n3.parray() };

            let item_type = variant_type as u32 & 0xff;

            dbg!(item_type);

            if item_type == VT_BSTR {
                let data = get_string_array(*array)?;

                return Ok(Variant::Array(
                    data.into_iter().map(|s| Variant::String(s)).collect(),
                ));
            }

            unimplemented!()
        }

        // See https://msdn.microsoft.com/en-us/library/cc237865.aspx for more info.
        // Rust can infer the return type of `vt.*Val()` calls,
        // but it's easier to read when the type is named explicitly.
        let variant_value = match variant_type as u32 {
            VT_BSTR => {
                let bstr_ptr: &BSTR = unsafe { vt.n1.n2().n3.bstrVal() };

                let prop_val: &WideCStr = unsafe { WideCStr::from_ptr_str(*bstr_ptr) };

                let property_value_as_string = prop_val.to_string()?;

                Variant::String(property_value_as_string)
            }
            VT_I2 => {
                let num: &i16 = unsafe { vt.n1.n2().n3.iVal() };

                Variant::I2(*num)
            }
            VT_I4 => {
                let num: &i32 = unsafe { vt.n1.n2().n3.lVal() };

                Variant::I4(*num)
            }
            VT_BOOL => {
                let value: &i16 = unsafe { vt.n1.n2().n3.boolVal() };

                match *value {
                    VARIANT_FALSE => Variant::Bool(false),
                    VARIANT_TRUE => Variant::Bool(true),
                    _ => bail!("Invalid bool value: {:#X}", value),
                }
            }
            VT_UI1 => {
                let num: &i8 = unsafe { vt.n1.n2().n3.cVal() };

                Variant::UI1(*num as u8)
            }
            VT_EMPTY => Variant::Empty,
            VT_NULL => Variant::Null,
            _ => bail!(
                "Converting from variant type {:#X} is not implemented yet",
                variant_type
            ),
        };

        debug!("Got {:?}", variant_value);

        Ok(variant_value)
    }
}

struct SeqAccess {
    data: Vec<Variant>,
    i: usize,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = crate::error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.i >= self.data.len() {
            return Ok(None);
        }

        let res: Variant = self.data.swap_remove(self.i);

        self.i += 1;

        seed.deserialize(res).map(Some)
    }
}

impl<'de> serde::Deserializer<'de> for Variant {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Variant::Null => visitor.visit_none(),
            Variant::Empty => visitor.visit_none(),
            Variant::String(s) => visitor.visit_string(s),
            Variant::I2(n) => visitor.visit_i16(n),
            Variant::I4(n) => visitor.visit_i32(n),
            Variant::I8(n) => visitor.visit_i64(n),
            Variant::Bool(b) => visitor.visit_bool(b),
            Variant::UI1(n) => visitor.visit_u8(n),
            Variant::UI8(n) => visitor.visit_u64(n),
            Variant::Array(v) => visitor.visit_seq(SeqAccess { data: v, i: 0 }),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
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
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Variant::Array(vec))
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
