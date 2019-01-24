use crate::{
    connection::WMIConnection,
    consts::{WBEM_FLAG_ALWAYS, WBEM_FLAG_NONSYSTEM_ONLY},
    safearray::{get_string_array, SafeArrayDestroy},
    utils::check_hres,
};
use failure::Error;
use log::debug;
use std::{ptr, ptr::Unique};
use widestring::WideCString;
use winapi::{
    shared::ntdef::{NULL},
    um::{
        oaidl::SAFEARRAY,
        wbemcli::{IEnumWbemClassObject, IWbemClassObject},
        wbemcli::{WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE},
    },
};

pub struct QueryResultEnumerator<'a> {
    wmi_con: &'a WMIConnection,
    p_enumerator: Option<Unique<IEnumWbemClassObject>>,
}

impl WMIConnection {
    /// Execute the given query and return n iterator for the results.
    pub fn raw_query(&self, query: impl AsRef<str>) -> Result<QueryResultEnumerator, Error> {
        let query_language = WideCString::from_str("WQL")?;
        let query = WideCString::from_str(query)?;

        let mut p_enumerator = NULL as *mut IEnumWbemClassObject;

        unsafe {
            check_hres((*self.svc()).ExecQuery(
                query_language.as_ptr() as *mut _,
                query.as_ptr() as *mut _,
                (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY) as i32,
                ptr::null_mut(),
                &mut p_enumerator,
            ))?;
        }

        debug!("Got enumerator {:?}", p_enumerator);

        Ok(QueryResultEnumerator {
            wmi_con: self,
            p_enumerator: Unique::new(p_enumerator),
        })
    }
}

impl<'a> QueryResultEnumerator<'a> {
    pub fn p(&self) -> *mut IEnumWbemClassObject {
        self.p_enumerator.unwrap().as_ptr()
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
                WBEM_FLAG_ALWAYS | WBEM_FLAG_NONSYSTEM_ONLY,
                ptr::null_mut(),
                &mut p_names,
            ))
        }?;

        let res = get_string_array(p_names);

        unsafe {
            check_hres(SafeArrayDestroy(p_names))?;
        }

        res
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

        debug!(
            "Got enumerator {:?} and obj {:?}",
            self.p_enumerator, pcls_obj
        );

        let pcls_wrapper = IWbemClassWrapper::new(Unique::new(pcls_obj));

        Some(Ok(pcls_wrapper))
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

        let enumerator = wmi_con
            .raw_query("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();
            let mut props = w.list_properties().unwrap();

            props.sort();

            assert_eq!(props.len(), 64);
            assert_eq!(props[..2], ["BootDevice", "BuildNumber"]);
            assert_eq!(props[props.len() - 2..], ["Version", "WindowsDirectory"])
        }
    }
}
