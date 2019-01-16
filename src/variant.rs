use failure::{bail, Error};
use log::debug;
use widestring::WideCStr;

use serde::de;
use serde::de::IntoDeserializer;

use winapi::shared::wtypes::*;
use winapi::um::oaidl::{VARIANT_n3, VARIANT};

#[derive(Debug)]
pub enum Variant {
    Null,

    String(String),

    I2(i16),
}

impl Variant {
    pub fn from_variant(vt: VARIANT) -> Result<Variant, Error> {
        let variant_type: VARTYPE = unsafe { vt.n1.n2().vt };

        println!("{:?}", variant_type);

        let variant_value = match variant_type as u32 {
            VT_BSTR => {
                let p = unsafe { vt.n1.n2().n3.bstrVal() };

                let prop_val: &WideCStr = unsafe { WideCStr::from_ptr_str(*p) };

                let property_value_as_string = prop_val.to_string()?;

                Variant::String(property_value_as_string)
            }
            VT_I2 => {
                let p: &i16 = unsafe { vt.n1.n2().n3.iVal() };

                Variant::I2(*p)
            }
            _ => bail!("Not implemented yet"),
        };



        debug!("Got {:?}", variant_value);

        Ok(variant_value)
    }

}
