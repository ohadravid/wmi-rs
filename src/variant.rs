use crate::{
    result_enumerator::IWbemClassWrapper, safearray::safe_array_to_vec, WMIError, WMIResult,
};
use serde::Serialize;
use std::convert::TryFrom;
use windows::core::{IUnknown, Interface, VARIANT};
use windows::Win32::Foundation::{VARIANT_BOOL, VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Variant::*;
use windows::Win32::System::Wmi::{self, IWbemClassObject, CIMTYPE_ENUMERATION};

#[derive(Clone, Debug, PartialEq, Serialize)]
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
        if $cim_type == Wmi::CIM_UINT8 {
            Ok(Variant::UI1($var as u8))
        } else if $cim_type == Wmi::CIM_UINT16 {
            Ok(Variant::UI2($var as u16))
        } else if $cim_type == Wmi::CIM_UINT32 {
            Ok(Variant::UI4($var as u32))
        } else if $cim_type == Wmi::CIM_UINT64 {
            Ok(Variant::UI8($var as u64))
        } else if $cim_type == Wmi::CIM_SINT8 {
            Ok(Variant::I1($var as i8))
        } else if $cim_type == Wmi::CIM_SINT16 {
            Ok(Variant::I2($var as i16))
        } else if $cim_type == Wmi::CIM_SINT32 {
            Ok(Variant::I4($var as i32))
        } else if $cim_type == Wmi::CIM_SINT64 {
            Ok(Variant::I8($var as i64))
        } else if $cim_type == Wmi::CIM_REAL32 {
            Ok(Variant::R4($var as f32))
        } else if $cim_type == Wmi::CIM_REAL64 {
            Ok(Variant::R8($var as f64))
        } else if $cim_type == Wmi::CIM_CHAR16 {
            Ok(Variant::String(String::from_utf16(&[$var as u16])?))
        } else {
            Err(WMIError::ConvertVariantError(format!(
                "Value {:?} cannot be turned into a CIMTYPE {:?}",
                $var, $cim_type,
            )))
        }
    };
}

impl Variant {
    /// Create a `Variant` instance from a raw `VARIANT`.
    ///
    /// Note: this function is safe since manipulating a `VARIANT` by hand is an *unsafe* operation,
    /// so we can assume that the `VARIANT` is valid.
    pub fn from_variant(variant: &VARIANT) -> WMIResult<Variant> {
        let vt = variant.as_raw();
        let variant_type = unsafe { vt.Anonymous.Anonymous.vt };

        // variant_type has two 'forms':
        // 1. A simple type like `VT_BSTR` .
        // 2. An array of certain type like `VT_ARRAY | VT_BSTR`.
        if variant_type & VT_ARRAY.0 == VT_ARRAY.0 {
            let array = unsafe {
                vt.Anonymous.Anonymous.Anonymous.parray
                    as *const windows::Win32::System::Com::SAFEARRAY
            };

            let item_type = VARENUM(variant_type & VT_TYPEMASK.0);

            return Ok(Variant::Array(unsafe {
                safe_array_to_vec(&*array, item_type)?
            }));
        }

        // See https://msdn.microsoft.com/en-us/library/cc237865.aspx for more info.
        // Rust can infer the return type of `vt.*Val()` calls,
        // but it's easier to read when the type is named explicitly.
        let variant_value = match VARENUM(variant_type) {
            VT_BSTR => Variant::String(variant.to_string()),
            VT_I1 => {
                let num = unsafe { vt.Anonymous.Anonymous.Anonymous.cVal };

                Variant::I1(num as _)
            }
            VT_I2 => {
                let num: i16 = unsafe { vt.Anonymous.Anonymous.Anonymous.iVal };

                Variant::I2(num)
            }
            VT_I4 => {
                let num: i32 = unsafe { vt.Anonymous.Anonymous.Anonymous.lVal };

                Variant::I4(num)
            }
            VT_I8 => {
                let num: i64 = unsafe { vt.Anonymous.Anonymous.Anonymous.llVal };

                Variant::I8(num)
            }
            VT_R4 => {
                let num: f32 = unsafe { vt.Anonymous.Anonymous.Anonymous.fltVal };

                Variant::R4(num)
            }
            VT_R8 => {
                let num: f64 = unsafe { vt.Anonymous.Anonymous.Anonymous.dblVal };

                Variant::R8(num)
            }
            VT_BOOL => {
                let value = unsafe { vt.Anonymous.Anonymous.Anonymous.boolVal };

                match VARIANT_BOOL(value) {
                    VARIANT_FALSE => Variant::Bool(false),
                    VARIANT_TRUE => Variant::Bool(true),
                    _ => return Err(WMIError::ConvertBoolError(value)),
                }
            }
            VT_UI1 => {
                let num: u8 = unsafe { vt.Anonymous.Anonymous.Anonymous.bVal };

                Variant::UI1(num)
            }
            VT_UI2 => {
                let num: u16 = unsafe { vt.Anonymous.Anonymous.Anonymous.uiVal };

                Variant::UI2(num)
            }
            VT_UI4 => {
                let num: u32 = unsafe { vt.Anonymous.Anonymous.Anonymous.ulVal };

                Variant::UI4(num)
            }
            VT_UI8 => {
                let num: u64 = unsafe { vt.Anonymous.Anonymous.Anonymous.ullVal };

                Variant::UI8(num)
            }
            VT_EMPTY => Variant::Empty,
            VT_NULL => Variant::Null,
            VT_UNKNOWN => {
                let ptr = unsafe {
                    IUnknown::from_raw_borrowed(&vt.Anonymous.Anonymous.Anonymous.punkVal)
                };
                let ptr = ptr.cloned().ok_or(WMIError::NullPointerResult)?;
                Variant::Unknown(IUnknownWrapper::new(ptr))
            }
            _ => return Err(WMIError::ConvertError(variant_type)),
        };

        Ok(variant_value)
    }

    /// Convert the variant it to a specific type.
    pub fn convert_into_cim_type(self, cim_type: CIMTYPE_ENUMERATION) -> WMIResult<Self> {
        if cim_type == Wmi::CIM_EMPTY {
            return Ok(Variant::Null);
        }

        if (Wmi::CIM_FLAG_ARRAY.0 & cim_type.0) != 0 {
            /*
            "If the type is actually an array type,
            the CimBaseType MUST be combined by using the bitwise OR operation with the CimArrayFlag value (0x2000)
            that results in the most significant octet containing 0x20
            and the lower octet containing the value of the CimBaseType."
            */
            return match self {
                // If we got an array, we just need to convert it's elements.
                Variant::Array(arr) => Variant::Array(arr)
                    .convert_into_cim_type(CIMTYPE_ENUMERATION(cim_type.0 & 0xff)),
                Variant::Empty | Variant::Null => Ok(Variant::Array(vec![])),
                // If we didn't get an array, we need to convert the element, but also wrap it in an array.
                not_array => {
                    Ok(Variant::Array(vec![not_array.convert_into_cim_type(
                        CIMTYPE_ENUMERATION(cim_type.0 & 0xff),
                    )?]))
                }
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
                if cim_type == Wmi::CIM_BOOLEAN {
                    Variant::Bool(b)
                } else {
                    return Err(WMIError::ConvertVariantError(format!(
                        "A boolean Variant cannot be turned into a CIMTYPE {:?}",
                        cim_type,
                    )));
                }
            }
            Variant::String(s) => {
                match cim_type {
                    Wmi::CIM_STRING | Wmi::CIM_CHAR16 => Variant::String(s),
                    Wmi::CIM_REAL64 => Variant::R8(s.parse()?),
                    Wmi::CIM_REAL32 => Variant::R4(s.parse()?),
                    Wmi::CIM_UINT64 => Variant::UI8(s.parse()?),
                    Wmi::CIM_SINT64 => Variant::I8(s.parse()?),
                    Wmi::CIM_UINT32 => Variant::UI4(s.parse()?),
                    Wmi::CIM_SINT32 => Variant::I4(s.parse()?),
                    Wmi::CIM_UINT16 => Variant::UI2(s.parse()?),
                    Wmi::CIM_SINT16 => Variant::I2(s.parse()?),
                    Wmi::CIM_UINT8 => Variant::UI1(s.parse()?),
                    Wmi::CIM_SINT8 => Variant::I1(s.parse()?),
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
                if cim_type == Wmi::CIM_OBJECT {
                    Variant::Object(u.to_wbem_class_obj()?)
                } else {
                    return Err(WMIError::ConvertVariantError(format!(
                        "A unknown Variant cannot be turned into a CIMTYPE {:?}",
                        cim_type,
                    )));
                }
            }
            Variant::Object(o) => Variant::Object(o),
        };

        Ok(converted_variant)
    }
}

impl TryFrom<Variant> for VARIANT {
    type Error = WMIError;
    fn try_from(value: Variant) -> WMIResult<VARIANT> {
        match value {
            Variant::Empty => Ok(VARIANT::new()),

            Variant::String(string) => Ok(VARIANT::from(string.as_str())),
            Variant::I1(int8) => Ok(VARIANT::from(int8)),
            Variant::I2(int16) => Ok(VARIANT::from(int16)),
            Variant::I4(int32) => Ok(VARIANT::from(int32)),
            Variant::I8(int64) => Ok(VARIANT::from(int64)),

            Variant::R4(float32) => Ok(VARIANT::from(float32)),
            Variant::R8(float64) => Ok(VARIANT::from(float64)),

            Variant::Bool(b) => Ok(VARIANT::from(b)),

            Variant::UI1(uint8) => Ok(VARIANT::from(uint8)),
            Variant::UI2(uint16) => Ok(VARIANT::from(uint16)),
            Variant::UI4(uint32) => Ok(VARIANT::from(uint32)),
            Variant::UI8(uint64) => Ok(VARIANT::from(uint64)),

            // windows-rs' VARIANT does not support creating these types of VARIANT at present
            Variant::Null => Err(WMIError::ConvertVariantError(
                "Cannot convert Variant::Null to a Windows VARIANT".to_string(),
            )),
            Variant::Array(_) => Err(WMIError::ConvertVariantError(
                "Cannot convert Variant::Array to a Windows VARIANT".to_string(),
            )),
            Variant::Unknown(_) => Err(WMIError::ConvertVariantError(
                "Cannot convert Variant::Unknown to a Windows VARIANT".to_string(),
            )),
            Variant::Object(_) => Err(WMIError::ConvertVariantError(
                "Cannot convert Variant::Object to a Windows VARIANT".to_string(),
            )),
        }
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

macro_rules! impl_wrap_type {
    ($target_type:ty, $variant_type:ident) => {
        impl From<$target_type> for Variant {
            fn from(value: $target_type) -> Self {
                Variant::$variant_type(value)
            }
        }
    };
}

macro_rules! bidirectional_variant_convert {
    ($target_type:ty, $variant_type:ident) => {
        impl_try_from_variant!($target_type, $variant_type);
        impl_wrap_type!($target_type, $variant_type);
    };
}

bidirectional_variant_convert!(String, String);
bidirectional_variant_convert!(i8, I1);
bidirectional_variant_convert!(i16, I2);
bidirectional_variant_convert!(i32, I4);
bidirectional_variant_convert!(i64, I8);
bidirectional_variant_convert!(u8, UI1);
bidirectional_variant_convert!(u16, UI2);
bidirectional_variant_convert!(u32, UI4);
bidirectional_variant_convert!(u64, UI8);
bidirectional_variant_convert!(f32, R4);
bidirectional_variant_convert!(f64, R8);
bidirectional_variant_convert!(bool, Bool);

impl From<()> for Variant {
    fn from(_value: ()) -> Self {
        Variant::Empty
    }
}

impl TryFrom<Variant> for () {
    type Error = WMIError;

    fn try_from(value: Variant) -> Result<(), Self::Error> {
        match value {
            Variant::Empty => Ok(()),
            other => Err(WMIError::ConvertVariantError(format!(
                "Variant {:?} cannot be turned into a {}",
                &other,
                stringify!(())
            ))),
        }
    }
}

/// A wrapper around the [`IUnknown`] interface. \
/// Used to retrive [`IWbemClassObject`][winapi::um::Wmi::IWbemClassObject]
///
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IUnknownWrapper {
    inner: IUnknown,
}

impl IUnknownWrapper {
    /// Wrapps around a non-null pointer to IUnknown
    ///
    pub fn new(ptr: IUnknown) -> Self {
        IUnknownWrapper { inner: ptr }
    }

    pub fn to_wbem_class_obj(&self) -> WMIResult<IWbemClassWrapper> {
        Ok(IWbemClassWrapper {
            inner: self.inner.cast::<IWbemClassObject>()?,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_convert_into_cim_type_sint8() {
        let cim_type = Wmi::CIM_SINT8;
        let variant = Variant::I1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I1(1));

        let variant = Variant::UI1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::I1(1));
    }

    #[test]
    fn it_convert_into_cim_type_uint8() {
        let cim_type = Wmi::CIM_UINT8;
        let variant = Variant::UI1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI1(1));

        let variant = Variant::I1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI1(1));
    }

    #[test]
    fn it_convert_into_cim_type_sint16() {
        let cim_type = Wmi::CIM_UINT16;
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
        let cim_type = Wmi::CIM_UINT32;
        let variant = Variant::I8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI4(1));

        let variant = Variant::UI8(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::UI4(1));
    }

    #[test]
    fn it_convert_into_cim_type_sint32() {
        let cim_type = Wmi::CIM_SINT32;
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
        let cim_type = Wmi::CIM_UINT64;
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
        let cim_type = Wmi::CIM_SINT64;
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
        let cim_type = Wmi::CIM_REAL32;
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
        let cim_type = Wmi::CIM_REAL64;
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
        let cim_type = Wmi::CIM_CHAR16;
        let variant = Variant::UI2(67);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::String("C".to_string()));
    }

    #[test]
    fn it_convert_into_cim_type_datetime() {
        let cim_type = Wmi::CIM_DATETIME;
        let datetime = "19980401135809.000000+000";
        let variant = Variant::String(datetime.to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::String(datetime.to_string()));
    }

    #[test]
    fn it_convert_into_cim_type_reference() {
        let cim_type = Wmi::CIM_REFERENCE;
        let datetime =
            r#"\\\\PC\\root\\cimv2:Win32_DiskDrive.DeviceID=\"\\\\\\\\.\\\\PHYSICALDRIVE0\""#;
        let variant = Variant::String(datetime.to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::String(datetime.to_string()));
    }

    #[test]
    fn it_convert_an_array_into_cim_type_array() {
        let cim_type = CIMTYPE_ENUMERATION(Wmi::CIM_UINT64.0 | Wmi::CIM_FLAG_ARRAY.0);
        let variant = Variant::Array(vec![Variant::String("1".to_string())]);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI8(1)]));

        let cim_type = CIMTYPE_ENUMERATION(Wmi::CIM_UINT8.0 | Wmi::CIM_FLAG_ARRAY.0);
        let variant = Variant::Array(vec![Variant::UI1(1)]);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI1(1)]));
    }

    #[test]
    fn it_convert_a_single_value_into_cim_type_array() {
        let cim_type = CIMTYPE_ENUMERATION(Wmi::CIM_UINT64.0 | Wmi::CIM_FLAG_ARRAY.0);
        let variant = Variant::String("1".to_string());
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI8(1)]));

        let cim_type = CIMTYPE_ENUMERATION(Wmi::CIM_UINT8.0 | Wmi::CIM_FLAG_ARRAY.0);
        let variant = Variant::UI1(1);
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![Variant::UI1(1)]));
    }

    #[test]
    fn it_convert_an_empty_into_cim_type_array() {
        let cim_type = CIMTYPE_ENUMERATION(Wmi::CIM_STRING.0 | Wmi::CIM_FLAG_ARRAY.0);
        let variant = Variant::Null;
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![]));

        let variant = Variant::Empty;
        let converted = variant.convert_into_cim_type(cim_type).unwrap();
        assert_eq!(converted, Variant::Array(vec![]));
    }

    #[test]
    fn it_bidirectional_string_convert() {
        let string = "Test String".to_string();
        let variant = Variant::from(string.clone());
        assert_eq!(
            <Variant as TryInto<String>>::try_into(variant).unwrap(),
            string
        );

        let variant = Variant::from(string.clone());
        let ms_variant = VARIANT::try_from(variant).unwrap();
        let variant = Variant::from(string.clone());
        assert_eq!(Variant::from_variant(&ms_variant).unwrap(), variant);
    }

    #[test]
    fn it_bidirectional_empty_convert() {
        let variant = Variant::from(());
        assert_eq!(<Variant as TryInto<()>>::try_into(variant).unwrap(), ());

        let variant = Variant::from(());
        let ms_variant = VARIANT::try_from(variant).unwrap();
        let variant = Variant::from(());
        assert_eq!(Variant::from_variant(&ms_variant).unwrap(), variant);
    }

    #[test]
    fn it_bidirectional_r8_convert() {
        let num = 0.123456789;
        let variant = Variant::from(num);
        assert_eq!(<Variant as TryInto<f64>>::try_into(variant).unwrap(), num);

        let variant = Variant::from(num);
        let ms_variant = VARIANT::try_from(variant).unwrap();
        let variant = Variant::from(num);
        assert_eq!(Variant::from_variant(&ms_variant).unwrap(), variant);
    }
}
