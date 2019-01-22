use crate::query::IWbemClassWrapper;
use chrono::prelude::*;
use failure::{bail, format_err};
use log::{debug, info};
use serde::de::Error as _;
use serde::de::{
    self, Deserialize, DeserializeSeed, IntoDeserializer, MapAccess, Unexpected, Visitor,
};
use std::iter::Peekable;
use std::mem;
use std::ptr;
use widestring::WideCStr;
use widestring::WideCString;
use winapi::um::oaidl::{VARIANT_n3, VARIANT};
use winapi::um::oleauto::VariantClear;

use crate::error::Error;
use crate::variant::Variant;
use std::fmt;
use std::str::FromStr;
use winapi::shared::wtypes::VARTYPE;

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    pub wbem_class_obj: &'de IWbemClassWrapper,
}

impl<'de> Deserializer<'de> {
    pub fn from_wbem_class_obj(wbem_class_obj: &'de IWbemClassWrapper) -> Self {
        Deserializer { wbem_class_obj }
    }
}

pub fn from_wbem_class_obj<'a, T>(wbem_class_obj: &'a IWbemClassWrapper) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_wbem_class_obj(wbem_class_obj);
    let t = T::deserialize(&mut deserializer)?;

    Ok(t)
}

struct WMIMapAccess<'a, 'de, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    fields: Peekable<I>,
    de: &'a Deserializer<'de>,
}

impl<'a, 'de, S, I> WMIMapAccess<'a, 'de, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    pub fn new(fields: I, de: &'a Deserializer<'de>) -> Self {
        Self {
            fields: fields.peekable(),
            de,
        }
    }
}

impl<'de, 'a, S, I> MapAccess<'de> for WMIMapAccess<'a, 'de, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(field) = self.fields.peek() {
            seed.deserialize(field.as_ref().into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let current_field = self
            .fields
            .next()
            .ok_or(format_err!("Expected current field to not be None"))?;

        let name_prop = WideCString::from_str(current_field).map_err(Error::from_err)?;

        let mut vt_prop: VARIANT = unsafe { mem::zeroed() };

        unsafe {
            (*self.de.wbem_class_obj.inner.unwrap().as_ptr()).Get(
                name_prop.as_ptr() as *mut _,
                0,
                &mut vt_prop,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }

        let property_value = Variant::from_variant(vt_prop)?;

        unsafe { VariantClear(&mut vt_prop) };

        match property_value {
            Variant::Null => unimplemented!(),
            Variant::Empty => unimplemented!(),
            Variant::String(s) => seed.deserialize(s.into_deserializer()),
            Variant::I2(n) => seed.deserialize(n.into_deserializer()),
            Variant::I4(n) => seed.deserialize(n.into_deserializer()),
            Variant::Bool(b) => seed.deserialize(b.into_deserializer()),
            Variant::UI1(n) => seed.deserialize(n.into_deserializer()),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let fields = self.wbem_class_obj.list_properties()?;

        visitor.visit_map(WMIMapAccess::new(fields.iter(), &self))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        println!("{:?} {:?}", fields, name);

        visitor.visit_map(WMIMapAccess::new(fields.iter(), &self))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod tests {
    use super::*;
    use crate::connection::COMLibrary;
    use crate::connection::WMIConnection;
    use crate::datetime::WMIDateTime;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        let p_svc = wmi_con.svc();

        assert_eq!(p_svc.is_null(), false);

        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
            Name: String,
            CurrentTimeZone: i16,
            Debug: bool,

            // This actually returns as an i32 from COM.
            EncryptionLevel: u32,
            ForegroundApplicationBoost: u8,

            LastBootUpTime: WMIDateTime,
        }

        let enumerator = wmi_con
            .query("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();

            let w: Win32_OperatingSystem = from_wbem_class_obj(&w).unwrap();

            println!("I am {:?}", w);
            assert_eq!(w.Caption, "Microsoft Windows 10 Pro");
            assert_eq!(
                w.Name,
                "Microsoft Windows 10 Pro|C:\\WINDOWS|\\Device\\Harddisk0\\Partition3"
            );
            assert_eq!(w.CurrentTimeZone, 60);
            assert_eq!(w.Debug, false);
            assert_eq!(w.EncryptionLevel, 256);
            assert_eq!(w.ForegroundApplicationBoost, 2);
            assert_eq!(
                w.LastBootUpTime.0.timezone().local_minus_utc() / 60,
                w.CurrentTimeZone as i32
            );
        }
    }

    #[test]
    fn it_desr_into_map() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        let p_svc = wmi_con.svc();

        assert_eq!(p_svc.is_null(), false);

        let enumerator = wmi_con
            .query("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();

            let w: HashMap<String, Variant> = from_wbem_class_obj(&w).unwrap();

            println!("I am {:?}", w);
            //            assert_eq!(w.Caption, "Microsoft Windows 10 Pro");
            //            assert_eq!(
            //                w.Name,
            //                "Microsoft Windows 10 Pro|C:\\WINDOWS|\\Device\\Harddisk0\\Partition3"
            //            );
            //            assert_eq!(w.CurrentTimeZone, 60);
            //            assert_eq!(w.Debug, false);
            //            assert_eq!(w.EncryptionLevel, 256);
            //            assert_eq!(w.ForegroundApplicationBoost, 2);
            //            assert_eq!(w.LastBootUpTime.0.timezone().local_minus_utc() / 60, w.CurrentTimeZone as i32);
        }
    }
}
