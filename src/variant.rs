use failure::{bail, Error};
use log::debug;
use widestring::WideCStr;

use serde::de;
use serde::de::IntoDeserializer;

use winapi::shared::wtypes::*;
use winapi::um::oaidl::{VARIANT_n3, VARIANT};

// See: https://msdn.microsoft.com/en-us/library/cc237864.aspx
const VARIANT_FALSE: i16 = 0x0000;

#[derive(Debug)]
pub enum Variant {
    Null,

    String(String),

    I2(i16),

    Bool(bool)
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
            }
            VT_I2 => {
                let num: &i16 = unsafe { vt.n1.n2().n3.iVal() };

                Variant::I2(*num)
            }
            VT_BOOL => {
                let value: &i16 = unsafe { vt.n1.n2().n3.boolVal() };

                match *value {
                    VARIANT_FALSE => Variant::Bool(false),
                    VARIANT_TRUE => Variant::Bool(true),
                    _ => bail!("Invalid bool value: {}", value)
                }

            }
            _ => bail!("Not implemented yet"),
        };



        debug!("Got {:?}", variant_value);

        Ok(variant_value)
    }

}
