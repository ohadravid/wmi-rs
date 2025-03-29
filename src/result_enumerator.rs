use crate::{
    connection::WMIConnection, de::wbem_class_de::from_wbem_class_obj,
    safearray::safe_array_to_vec_of_strings, Variant, WMIError, WMIResult,
};
use log::trace;
use serde::{
    de,
    ser::{Error, SerializeMap},
    Serialize,
};
use std::ptr;
use windows::Win32::System::Ole::SafeArrayDestroy;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::{
    IEnumWbemClassObject, IWbemClassObject, CIMTYPE_ENUMERATION, WBEM_FLAG_ALWAYS,
    WBEM_FLAG_NONSYSTEM_ONLY, WBEM_INFINITE,
};
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::System::Wmi::WBEM_CONDITION_FLAG_TYPE,
};

/// A wrapper around a raw pointer to IWbemClassObject, which also takes care of releasing
/// the object when dropped.
///
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IWbemClassWrapper {
    pub inner: IWbemClassObject,
}

impl IWbemClassWrapper {
    pub fn new(inner: IWbemClassObject) -> Self {
        Self { inner }
    }

    /// Return the names of all the properties of the given object.
    ///
    pub fn list_properties(&self) -> WMIResult<Vec<String>> {
        let p_names = unsafe {
            self.inner.GetNames(
                None,
                WBEM_CONDITION_FLAG_TYPE(WBEM_FLAG_ALWAYS.0 | WBEM_FLAG_NONSYSTEM_ONLY.0),
                ptr::null_mut(),
            )
        }?;

        let res = unsafe { safe_array_to_vec_of_strings(unsafe { &*p_names }) };

        unsafe { SafeArrayDestroy(p_names) }?;

        res
    }

    pub fn get_property(&self, property_name: &str) -> WMIResult<Variant> {
        let name_prop = HSTRING::from(property_name);

        let mut vt_prop = VARIANT::default();

        let mut cim_type = 0;

        unsafe {
            self.inner.Get(
                PCWSTR::from_raw(name_prop.as_ptr()),
                0,
                &mut vt_prop,
                Some(&mut cim_type),
                None,
            )?;

            let property_value = Variant::from_variant(&vt_prop)?
                .convert_into_cim_type(CIMTYPE_ENUMERATION(cim_type))?;

            Ok(property_value)
        }
    }

    pub fn put_property(&self, property_name: &str, value: impl Into<Variant>) -> WMIResult<()> {
        let name_prop = HSTRING::from(property_name);

        let value = value.into();
        let vt_prop: VARIANT = value.try_into()?;

        // "In every other case, vtType must be 0 (zero)"
        // See more at https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemclassobject-put
        let cim_type = 0;

        unsafe {
            self.inner
                .Put(PCWSTR::from_raw(name_prop.as_ptr()), 0, &vt_prop, cim_type)?;
        }

        Ok(())
    }

    pub fn path(&self) -> WMIResult<String> {
        self.get_property("__Path").and_then(Variant::try_into)
    }

    pub fn class(&self) -> WMIResult<String> {
        self.get_property("__Class").and_then(Variant::try_into)
    }

    pub fn into_desr<T>(self) -> WMIResult<T>
    where
        T: de::DeserializeOwned,
    {
        from_wbem_class_obj(self)
    }
}

impl Serialize for IWbemClassWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let properties = self.list_properties().map_err(Error::custom)?;
        let mut s = serializer.serialize_map(Some(properties.len()))?;
        for property in properties.iter() {
            let value = self.get_property(property).unwrap();
            s.serialize_entry(property, &value)?;
        }
        s.end()
    }
}

pub struct QueryResultEnumerator<'a> {
    _wmi_con: &'a WMIConnection,
    p_enumerator: IEnumWbemClassObject,
}

impl<'a> QueryResultEnumerator<'a> {
    pub fn new(wmi_con: &'a WMIConnection, p_enumerator: IEnumWbemClassObject) -> Self {
        Self {
            _wmi_con: wmi_con,
            p_enumerator,
        }
    }
}

impl<'a> Iterator for QueryResultEnumerator<'a> {
    type Item = WMIResult<IWbemClassWrapper>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut objs = [None; 1];
        let mut return_value = 0;

        let res = unsafe {
            self.p_enumerator
                .Next(WBEM_INFINITE, &mut objs, &mut return_value)
        };

        if let Err(e) = res.ok() {
            return Some(Err(e.into()));
        }

        if return_value == 0 {
            return None;
        }

        trace!(
            "Got enumerator {:?} and obj {:?}",
            self.p_enumerator,
            &objs[0]
        );

        let [obj] = objs;
        let pcls_ptr = obj.ok_or(WMIError::NullPointerResult);

        match pcls_ptr {
            Err(e) => Some(Err(e)),
            Ok(pcls_ptr) => Some(Ok(IWbemClassWrapper::new(pcls_ptr))),
        }
    }
}
