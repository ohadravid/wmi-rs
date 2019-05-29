use crate::safearray::safe_array_to_vec;
use failure::{bail, Error, format_err};
use widestring::WideCStr;
use winapi::{
    shared::wtypes::*,
    um::{oaidl::SAFEARRAY, oaidl::VARIANT},
};
use std::convert::TryFrom;

// See: https://msdn.microsoft.com/en-us/library/cc237864.aspx
const VARIANT_FALSE: i16 = 0x0000;

#[derive(Debug, PartialEq, Hash)]
pub enum Variant {
    Empty,
    Null,

    String(String),

    I1(i8),
    I2(i16),
    I4(i32),
    I8(i64),

    Bool(bool),

    UI1(u8),
    UI2(u16),
    UI4(u32),
    UI8(u64),

    Array(Vec<Variant>),
}

impl Variant {
    pub fn from_variant(vt: VARIANT) -> Result<Variant, Error> {
        let variant_type: VARTYPE = unsafe { vt.n1.n2().vt };

        // variant_type has two 'forms':
        // 1. A simple type like `VT_BSTR` .
        // 2. An array of certain type like `VT_ARRAY | VT_BSTR`.
        if variant_type as u32 & VT_ARRAY == VT_ARRAY {
            let array: &*mut SAFEARRAY = unsafe { vt.n1.n2().n3.parray() };

            let item_type = variant_type as u32 & VT_TYPEMASK;

            return Ok(Variant::Array(safe_array_to_vec(*array, item_type as u32)?));
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
            VT_I1 => {
                let num: &i8 = unsafe { vt.n1.n2().n3.cVal() };

                Variant::I1(*num)
            }
            VT_I2 => {
                let num: &i16 = unsafe { vt.n1.n2().n3.iVal() };

                Variant::I2(*num)
            }
            VT_I4 => {
                let num: &i32 = unsafe { vt.n1.n2().n3.lVal() };

                Variant::I4(*num)
            }
            VT_I8 => {
                let num: &i64 = unsafe { vt.n1.n2().n3.llVal() };

                Variant::I8(*num)
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
                let num: &u8 = unsafe { vt.n1.n2().n3.bVal() };

                Variant::UI1(*num)
            }
            VT_UI2 => {
                let num: &u16 = unsafe { vt.n1.n2().n3.uiVal() };

                Variant::UI2(*num)
            }
            VT_UI4 => {
                let num: &u32 = unsafe { vt.n1.n2().n3.ulVal() };

                Variant::UI4(*num)
            }
            VT_UI8 => {
                let num: &u64 = unsafe { vt.n1.n2().n3.ullVal() };

                Variant::UI8(*num)
            }
            VT_EMPTY => Variant::Empty,
            VT_NULL => Variant::Null,
            _ => bail!(
                "Converting from variant type {:#X} is not implemented yet",
                variant_type
            ),
        };

        Ok(variant_value)
    }
}

macro_rules! impl_try_from_variant {
    ($target_type:ty, $variant_type:ident) => {
        impl TryFrom<Variant> for $target_type {
            type Error = Error;

            fn try_from(value: Variant) -> Result<$target_type, Self::Error> {
                match value {
                    Variant::$variant_type(item) => Ok(item),
                    other => Err(format_err!("Variant {:?} cannot be turned into a {}", &other, stringify!($target_type)))
                }
            }
        }
    }
}

impl_try_from_variant!(String, String);
impl_try_from_variant!(i8, I1);
impl_try_from_variant!(i16, I2);
impl_try_from_variant!(i32, I4);
impl_try_from_variant!(i64, I8);
impl_try_from_variant!(u8, UI1);
impl_try_from_variant!(u16, UI2);
impl_try_from_variant!(u32, UI4);
impl_try_from_variant!(u64, UI8);
impl_try_from_variant!(bool, Bool);