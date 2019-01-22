use crate::connection::WMIConnection;
use crate::utils::check_hres;
use failure::Error;
use log::debug;
use std::mem;
use std::ptr;
use std::ptr::Unique;
use std::slice;
use widestring::WideCStr;
use widestring::WideCString;
use winapi::shared::minwindef::UINT;
use winapi::shared::ntdef::LONG;
use winapi::shared::ntdef::NULL;
use winapi::shared::rpcdce::RPC_C_AUTHN_LEVEL_CALL;
use winapi::shared::rpcdce::RPC_C_AUTHN_WINNT;
use winapi::shared::rpcdce::RPC_C_AUTHZ_NONE;
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypes::BSTR;
use winapi::um::oaidl::{VARIANT_n3, SAFEARRAY, VARIANT};
use winapi::um::oleauto::SafeArrayAccessData;
use winapi::um::oleauto::SafeArrayUnaccessData;
use winapi::um::oleauto::VariantClear;
use winapi::um::wbemcli::{
    CLSID_WbemLocator, IEnumWbemClassObject, IID_IWbemLocator, IWbemClassObject, IWbemLocator,
    IWbemServices,
};
use winapi::um::wbemcli::{WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE};

pub struct QueryResultEnumerator<'a> {
    wmi_con: &'a WMIConnection,
    p_enumerator: Option<Unique<IEnumWbemClassObject>>,
}

impl WMIConnection {
    pub fn query(&self, query: impl AsRef<str>) -> Result<QueryResultEnumerator, Error> {
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

pub struct IWbemClassWrapper {
    pub inner: Option<Unique<IWbemClassObject>>,
}

const WBEM_FLAG_ALWAYS: i32 = 0;
const WBEM_FLAG_ONLY_IF_TRUE: i32 = 0x1;
const WBEM_FLAG_ONLY_IF_FALSE: i32 = 0x2;
const WBEM_FLAG_ONLY_IF_IDENTICAL: i32 = 0x3;
const WBEM_MASK_PRIMARY_CONDITION: i32 = 0x3;
const WBEM_FLAG_KEYS_ONLY: i32 = 0x4;
const WBEM_FLAG_REFS_ONLY: i32 = 0x8;
const WBEM_FLAG_LOCAL_ONLY: i32 = 0x10;
const WBEM_FLAG_PROPAGATED_ONLY: i32 = 0x20;
const WBEM_FLAG_SYSTEM_ONLY: i32 = 0x30;
const WBEM_FLAG_NONSYSTEM_ONLY: i32 = 0x40;
const WBEM_MASK_CONDITION_ORIGIN: i32 = 0x70;
const WBEM_FLAG_CLASS_OVERRIDES_ONLY: i32 = 0x100;
const WBEM_FLAG_CLASS_LOCAL_AND_OVERRIDES: i32 = 0x200;
const WBEM_MASK_CLASS_CONDITION: i32 = 0x30;

extern "system" {
    pub fn SafeArrayGetLBound(psa: *mut SAFEARRAY, nDim: UINT, plLbound: *mut LONG) -> HRESULT;

    pub fn SafeArrayGetUBound(psa: *mut SAFEARRAY, nDim: UINT, plUbound: *mut LONG) -> HRESULT;

    pub fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
}

impl IWbemClassWrapper {
    pub fn new(ptr: Option<Unique<IWbemClassObject>>) -> Self {
        Self { inner: ptr }
    }

    pub fn list_properties(&self) -> Result<Vec<String>, Error> {
        let mut p_names = NULL as *mut SAFEARRAY;

        unsafe {
            check_hres((*self.inner.unwrap().as_ptr()).GetNames(
                ptr::null(),
                WBEM_FLAG_ALWAYS,
                ptr::null_mut(),
                &mut p_names,
            ))
        }?;

        let mut p_data = NULL; // as *mut BSTR;
        let mut lstart: i32 = 0;
        let mut lend: i32 = 0;

        unsafe {
            check_hres(SafeArrayGetLBound(p_names, 1, &mut lstart as _))?;
            check_hres(SafeArrayGetUBound(p_names, 1, &mut lend as _))?;
            check_hres(SafeArrayAccessData(p_names, &mut p_data))?;
        }

        dbg!(lstart);
        dbg!(lend);
        dbg!(p_data);

        let mut p_data: *mut BSTR = p_data as _;

        let mut data_slice = unsafe {
             slice::from_raw_parts(p_data, lend as usize)
        };

        for prop_name_bstr in data_slice {
            let prop_name: &WideCStr = unsafe { WideCStr::from_ptr_str(*prop_name_bstr) };

            let prop_name = prop_name.to_string()?;

            dbg!(prop_name);
        }

        unsafe {
            check_hres(SafeArrayUnaccessData(p_names))?;
            check_hres(SafeArrayDestroy(p_names))?;
        }

        unimplemented!();
    }
}

impl Drop for IWbemClassWrapper {
    fn drop(&mut self) {
        if let Some(pcls_obj) = self.inner {
            unsafe {
                (*pcls_obj.as_ptr()).Release();
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

        let res = unsafe {
            check_hres((*self.p_enumerator.unwrap().as_ptr()).Next(
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
            .query("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();

            assert_eq!(w.list_properties().unwrap(), vec!["Name"]);
        }
    }
}
