use crate::{
    result_enumerator::IWbemClassWrapper, safearray::safe_array_to_vec, utils::check_hres,
    WMIError, WMIResult,
};
use serde::Serialize;
use std::{convert::TryFrom, ptr::NonNull};
use widestring::WideCStr;
use winapi::{
    ctypes::c_void,
    shared::{ntdef::NULL, wtypes::*},
    um::{
        oaidl::SAFEARRAY,
        oaidl::VARIANT,
        unknwnbase::IUnknown,
        wbemcli::{self, IID_IWbemClassObject, CIMTYPE_ENUMERATION},
    },
};

// See: https://msdn.microsoft.com/en-us/library/cc237864.aspx
const VARIANT_FALSE: i16 = 0x0000;

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Variant {
    Empty,
    Null,

    String(String),

    I1(i8),
    I2(i16),
    I4(i32),
    I8(i64),

    R4(f32),
    R8(f64),

    Bool(bool),

    UI1(u8),
    UI2(u16),
    UI4(u32),
    UI8(u64),

    Array(Vec<Variant>),

    /// Temporary variant used internally
    Unknown(IUnknownWrapper),
    Object(IWbemClassWrapper),
}

// The `cast_num` macro is used to convert a numerical variable to a variant of the given CIMTYPE.
macro_rules! cast_num {
    ($var:ident, $cim_type: ident) => {
        if $cim_type == wbemcli::CIM_UINT8 {
            Ok(Variant::UI1($var as u8))
        } else if $cim_type == wbemcli::CIM_UINT16 {
            Ok(Variant::UI2($var as u16))
        } else if $cim_type == wbemcli::CIM_UINT32 {
            Ok(Variant::UI4($var as u32))
        } else if $cim_type == wbemcli::CIM_UINT64 {
            Ok(Variant::UI8($var as u64))
        } else if $cim_type == wbemcli::CIM_SINT8 {
            Ok(Variant::I1($var as i8))
        } else if $cim_type == wbemcli::CIM_SINT16 {
            Ok(Variant::I2($var as i16))
        } else if $cim_type == wbemcli::CIM_SINT32 {
            Ok(Variant::I4($var as i32))
        } else if $cim_type == wbemcli::CIM_SINT64 {
            Ok(Variant::I8($var as i64))
        } else if $cim_type == wbemcli::CIM_REAL32 {
            Ok(Variant::R4($var as f32))
        } else if $cim_type == wbemcli::CIM_REAL64 {
            Ok(Variant::R8($var as f64))
        } else if $cim_type == wbemcli::CIM_CHAR16 {
            Ok(Variant::String(String::from_utf16(&[$var as u16])?))
        } else {
            Err(WMIError::ConvertVariantError(format!(
                "Value {:?} cannot be turned into a CIMTYPE {}",
                $var, $cim_type,
            )))
        }
    };
}

impl Variant {
    /// Create a `Variant` instance from a raw `VARIANT`.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it is the caller's responsibility to ensure that the VARIANT is correctly initialized.
    pub unsafe fn from_variant(vt: VARIANT) -> WMIResult<Variant> {
        let variant_type: VARTYPE = unsafe { vt.n1.n2().vt };

        // variant_type has two 'forms':
        // 1. A simple type like `VT_BSTR` .
        // 2. An array of certain type like `VT_ARRAY | VT_BSTR`.
        if variant_type as u32 & VT_ARRAY == VT_ARRAY {
            let array: &*mut SAFEARRAY = unsafe { vt.n1.n2().n3.parray() };

            let item_type = variant_type as u32 & VT_TYPEMASK;

            return Ok(Variant::Array(unsafe {
                safe_array_to_vec(*array, item_type)?
            }));
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
            VT_R4 => {
                let num: &f32 = unsafe { vt.n1.n2().n3.fltVal() };

                Variant::R4(*num)
            }
            VT_R8 => {
                let num: &f64 = unsafe { vt.n1.n2().n3.dblVal() };

                Variant::R8(*num)
            }
            VT_BOOL => {
                let value: &i16 = unsafe { vt.n1.n2().n3.boolVal() };

                match *value {
                    VARIANT_FALSE => Variant::Bool(false),
                    VARIANT_TRUE => Variant::Bool(true),
                    _ => return Err(WMIError::ConvertBoolError(*value)),
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
            VT_UNKNOWN => {
                let unk: &*mut IUnknown = unsafe { vt.n1.n2().n3.punkVal() };
                let ptr = NonNull::new(*unk).ok_or(WMIError::NullPointerResult)?;
                Variant::Unknown(unsafe { IUnknownWrapper::new(ptr) })
            }
            _ => return Err(WMIError::ConvertError(variant_type)),
        };

        Ok(variant_value)
    }

    /// Convert the variant it to a specific type.
    pub fn convert_into_cim_type(self, cim_type: CIMTYPE_ENUMERATION) -> WMIResult<Self> {
        if cim_type == wbemcli::CIM_EMPTY {
            return Ok(Variant::Null);
        }

        if wbemcli::CIM_FLAG_ARRAY & cim_type != 0 {
            /*
            "If the type is actually an array type,
            the CimBaseType MUST be combined by using the bitwise OR operation with the CimArrayFlag value (0x2000)
            that results in the most significant octet containing 0x20
            and the lower octet containing the value of the CimBaseType."
            */
            return match self {
                // If we got an array, we just need to convert it's elements.
                Variant::Array(arr) => Variant::Array(arr).convert_into_cim_type(cim_type & 0xff),
                Variant::Empty | Variant::Null => Ok(Variant::Array(vec![])),
                // If we didn't get an array, we need to convert the element, but also wrap it in an array.
                not_array => Ok(Variant::Array(vec![
                    not_array.convert_into_cim_type(cim_type & 0xff)?
                ])),
            };
        }

        // The `convert_into_cim_type` function is used to convert a `Variant` into a CIM-type.
        // we cannot use `try_into` because we need to support i8 to u8 conversion.
        let converted_variant = match self {
            Variant::Empty => Variant::Empty,
            Variant::Null => Variant::Null,
            Variant::I1(n) => cast_num!(n, cim_type)?,
            Variant::I2(n) => cast_num!(n, cim_type)?,
            Variant::I4(n) => cast_num!(n, cim_type)?,
            Variant::I8(n) => cast_num!(n, cim_type)?,
            Variant::R4(f) => cast_num!(f, cim_type)?,
            Variant::R8(f) => cast_num!(f, cim_type)?,
            Variant::UI1(n) => cast_num!(n, cim_type)?,
            Variant::UI2(n) => cast_num!(n, cim_type)?,
            Variant::UI4(n) => cast_num!(n, cim_type)?,
            Variant::UI8(n) => cast_num!(n, cim_type)?,
            Variant::Bool(b) => {
                if cim_type == wbemcli::CIM_BOOLEAN {
                    Variant::Bool(b)
                } else {
                    return Err(WMIError::ConvertVariantError(format!(
                        "A boolean Variant cannot be turned into a CIMTYPE {}",
                        &cim_type,
                    )));
                }
            }
            Variant::String(s) => {
                match cim_type {
                    wbemcli::CIM_STRING | wbemcli::CIM_CHAR16 => Variant::String(s),
                    wbemcli::CIM_REAL64 => Variant::R8(s.parse()?),
                    wbemcli::CIM_REAL32 => Variant::R4(s.parse()?),
                    wbemcli::CIM_UINT64 => Variant::UI8(s.parse()?),
                    wbemcli::CIM_SINT64 => Variant::I8(s.parse()?),
                    wbemcli::CIM_UINT32 => Variant::UI4(s.parse()?),
                    wbemcli::CIM_SINT32 => Variant::I4(s.parse()?),
                    wbemcli::CIM_UINT16 => Variant::UI2(s.parse()?),
                    wbemcli::CIM_SINT16 => Variant::I2(s.parse()?),
                    wbemcli::CIM_UINT8 => Variant::UI1(s.parse()?),
                    wbemcli::CIM_SINT8 => Variant::I1(s.parse()?),
                    // Since Variant cannot natively represent a CIM_DATETIME or a CIM_REFERENCE (or any other), we keep it as a string.
                    _ => Variant::String(s),
                }
            }
            Variant::Array(variants) => {
                let converted_variants = variants
                    .into_iter()
                    .map(|variant| variant.convert_into_cim_type(cim_type))
                    .collect::<Result<Vec<_>, WMIError>>()?;

                Variant::Array(converted_variants)
            }
            Variant::Unknown(u) => {
                if cim_type == wbemcli::CIM_OBJECT {
                    Variant::Object(u.to_wbem_class_obj()?)
                } else {
                    return Err(WMIError::ConvertVariantError(format!(
                        "A unknown Variant cannot be turned into a CIMTYPE {}",
                        &cim_type,
                    )));
                }
            }
            Variant::Object(o) => Variant::Object(o),
        };

        Ok(converted_variant)
    }
}

/// A wrapper around the [`IUnknown`] interface. \
/// Used to retrive [`IWbemClassObject`][winapi::um::wbemcli::IWbemClassObject]
///
#[derive(Debug, PartialEq, Eq)]
pub struct IUnknownWrapper {
    inner: NonNull<IUnknown>,
}

impl IUnknownWrapper {
    /// Wrapps around a non-null pointer to IUnknown
    ///
    pub unsafe fn new(ptr: NonNull<IUnknown>) -> Self {
        IUnknownWrapper { inner: ptr }
    }

    pub fn to_wbem_class_obj(&self) -> WMIResult<IWbemClassWrapper> {
        let ptr = self.inner.as_ptr();
        let mut obj_ptr = NULL as *mut c_void;

        unsafe {
            check_hres((*ptr).QueryInterface(&IID_IWbemClassObject, &mut obj_ptr))?;
        }

        let obj = NonNull::new(obj_ptr as *mut _).ok_or(WMIError::NullPointerResult)?;
        Ok(unsafe { IWbemClassWrapper::new(obj) })
    }
}

impl Serialize for IUnknownWrapper {
    /// IUnknownWrapper serializaes to `()`, since it should have been converted into [Variant::Object]
    ///
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

macro_rules! impl_try_from_variant {
    ($target_type:ty, $variant_type:ident) => {
        impl TryFrom<Variant> for $target_type {
            type Error = WMIError;

            fn try_from(value: Variant) -> Result<$target_type, Self::Error> {
                match value {
                    Variant::$variant_type(item) => Ok(item),
                    other => Err(WMIError::ConvertVariantError(format!(
                        "Variant {:?} cannot be turned into a {}",
                        &other,
                        stringify!($target_type)
                    ))),
                }
            }
        }
    };
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
impl_try_from_variant!(f32, R4);
impl_try_from_variant!(f64, R8);
impl_try_from_variant!(bool, Bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_convert_into_cim_type_sint8() {
        let cim_type = wbemcli::CIM_SINT8;
        let variant = Variant::I1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I1(1));

        let variant = Variant::UI1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I1(1));
    }

    #[test]
    fn it_convert_into_cim_type_uint8() {
        let cim_type = wbemcli::CIM_UINT8;
        let variant = Variant::UI1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI1(1));

        let variant = Variant::I1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI1(1));
    }

    #[test]
    fn it_convert_into_cim_type_sint16() {
        let cim_type = wbemcli::CIM_UINT16;
        let variant = Variant::I2(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI2(1));

        let variant = Variant::UI2(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI2(1));

        let variant = Variant::I1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI2(1));
    }

    #[test]
    fn it_convert_into_cim_type_uint32() {
        let cim_type = wbemcli::CIM_UINT32;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI4(1));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI4(1));
    }

    #[test]
    fn it_convert_into_cim_type_sint32() {
        let cim_type = wbemcli::CIM_SINT32;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I4(1));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I4(1));

        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I4(1));
    }

    #[test]
    fn it_convert_into_cim_type_uint64() {
        let cim_type = wbemcli::CIM_UINT64;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI8(1));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI8(1));

        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI8(1));
    }

    #[test]
    fn it_convert_into_cim_type_sint64() {
        let cim_type = wbemcli::CIM_SINT64;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I8(1));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I8(1));

        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I8(1));
    }

    #[test]
    fn it_convert_into_cim_type_real32() {
        let cim_type = wbemcli::CIM_REAL32;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R4(1.0));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R4(1.0));

        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R4(1.0));

        let variant = Variant::String("1.0".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R4(1.0));
    }

    #[test]
    fn it_convert_into_cim_type_real64() {
        let cim_type = wbemcli::CIM_REAL64;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R8(1.0));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R8(1.0));

        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R8(1.0));

        let variant = Variant::String("1.0".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::R8(1.0));
    }

    #[test]
    fn it_convert_into_cim_char16() {
        let cim_type = wbemcli::CIM_CHAR16;
        let variant = Variant::UI2(67);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::String("C".to_string()));
    }

    #[test]
    fn it_convert_into_cim_type_datetime() {
        let cim_type = wbemcli::CIM_DATETIME;
        let datetime = "19980401135809.000000+000";
        let variant = Variant::String(datetime.to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::String(datetime.to_string()));
    }

    #[test]
    fn it_convert_into_cim_type_reference() {
        let cim_type = wbemcli::CIM_REFERENCE;
        let datetime =
            r#"\\\\PC\\root\\cimv2:Win32_DiskDrive.DeviceID=\"\\\\\\\\.\\\\PHYSICALDRIVE0\""#;
        let variant = Variant::String(datetime.to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::String(datetime.to_string()));
    }

    #[test]
    fn it_convert_an_array_into_cim_type_array() {
        let cim_type = wbemcli::CIM_UINT64 | wbemcli::CIM_FLAG_ARRAY;
        let variant = Variant::Array(vec![Variant::String("1".to_string())]);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI8(1)]));

        let cim_type = wbemcli::CIM_UINT8 | wbemcli::CIM_FLAG_ARRAY;
        let variant = Variant::Array(vec![Variant::UI1(1)]);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI1(1)]));
    }

    #[test]
    fn it_convert_a_single_value_into_cim_type_array() {
        let cim_type = wbemcli::CIM_UINT64 | wbemcli::CIM_FLAG_ARRAY;
        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI8(1)]));

        let cim_type = wbemcli::CIM_UINT8 | wbemcli::CIM_FLAG_ARRAY;
        let variant = Variant::UI1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI1(1)]));
    }

    #[test]
    fn it_convert_an_empty_into_cim_type_array() {
        let cim_type = wbemcli::CIM_STRING | wbemcli::CIM_FLAG_ARRAY;
        let variant = Variant::Null;
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![]));

        let variant = Variant::Empty;
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![]));
    }
}
