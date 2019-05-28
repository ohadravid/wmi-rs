use crate::{
    connection::WMIConnection, safearray::safe_array_to_vec_of_strings, utils::check_hres,
};
use failure::Error;
use log::trace;
use std::{mem, ptr, ptr::Unique};
use widestring::WideCString;
use winapi::um::oaidl::VARIANT;
use winapi::um::oleauto::VariantClear;
use winapi::{
    shared::ntdef::NULL,
    um::{
        oaidl::SAFEARRAY,
        oleauto::SafeArrayDestroy,
        wbemcli::{
            IEnumWbemClassObject, IWbemClassObject, WBEM_FLAG_ALWAYS, WBEM_FLAG_NONSYSTEM_ONLY,
            WBEM_INFINITE,
        },
    },
};
use crate::variant::Variant;

/// A wrapper around a raw pointer to IWbemClassObject, which also takes care of releasing
/// the object when dropped.
///
#[derive(Debug)]
pub struct IWbemClassWrapper {
    pub inner: Option<Unique<IWbemClassObject>>,
}

impl IWbemClassWrapper {
    pub fn new(ptr: Option<Unique<IWbemClassObject>>) -> Self {
        Self { inner: ptr }
    }

    /// Return the names of all the properties of the given object.
    ///
    pub fn list_properties(&self) -> Result<Vec<String>, Error> {
        // This will store the properties names from the GetNames call.
        let mut p_names = NULL as *mut SAFEARRAY;

        let ptr = self.inner.unwrap().as_ptr();

        unsafe {
            check_hres((*ptr).GetNames(
                ptr::null(),
                (WBEM_FLAG_ALWAYS | WBEM_FLAG_NONSYSTEM_ONLY) as i32,
                ptr::null_mut(),
                &mut p_names,
            ))
        }?;

        let res = safe_array_to_vec_of_strings(p_names);

        unsafe {
            check_hres(SafeArrayDestroy(p_names))?;
        }

        res
    }

    pub fn get_property(&self, property_name: &str) -> Result<Variant, Error> {
        let name_prop = WideCString::from_str(property_name)?;

        let mut vt_prop: VARIANT = unsafe { mem::zeroed() };

        unsafe {
            (*self.inner.unwrap().as_ptr()).Get(
                name_prop.as_ptr() as *mut _,
                0,
                &mut vt_prop,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }

        let property_value = Variant::from_variant(vt_prop)?;

        unsafe { VariantClear(&mut vt_prop) };

        Ok(property_value)
    }
}

impl Drop for IWbemClassWrapper {
    fn drop(&mut self) {
        if let Some(pcls_obj) = self.inner {
            let ptr = pcls_obj.as_ptr();

            unsafe {
                (*ptr).Release();
            }
        }
    }
}

pub struct QueryResultEnumerator<'a> {
    wmi_con: &'a WMIConnection,
    p_enumerator: Option<Unique<IEnumWbemClassObject>>,
}

impl<'a> QueryResultEnumerator<'a> {
    pub fn new(wmi_con: &'a WMIConnection, p_enumerator: *mut IEnumWbemClassObject) -> Self {
        Self {
            wmi_con,
            p_enumerator: Unique::new(p_enumerator),
        }
    }
}

impl<'a> Drop for QueryResultEnumerator<'a> {
    fn drop(&mut self) {
        if let Some(p_enumerator) = self.p_enumerator {
            unsafe {
                (*p_enumerator.as_ptr()).Release();
            }
        }
    }
}

impl<'a> Iterator for QueryResultEnumerator<'a> {
    type Item = Result<IWbemClassWrapper, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pcls_obj = NULL as *mut IWbemClassObject;
        let mut return_value = 0;

        if self.p_enumerator.is_none() {
            return None;
        }

        let raw_enumerator_prt = self.p_enumerator.unwrap().as_ptr();

        let res = unsafe {
            check_hres((*raw_enumerator_prt).Next(
                WBEM_INFINITE as i32,
                1,
                &mut pcls_obj,
                &mut return_value,
            ))
        };

        if let Err(e) = res {
            return Some(Err(e.into()));
        }

        if return_value == 0 {
            return None;
        }

        trace!(
            "Got enumerator {:?} and obj {:?}",
            self.p_enumerator,
            pcls_obj
        );

        let pcls_wrapper = IWbemClassWrapper::new(Unique::new(pcls_obj));

        Some(Ok(pcls_wrapper))
    }
}
