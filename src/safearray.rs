use winapi::shared::ntdef::NULL;
use crate::utils::check_hres;
use std::slice;
use winapi::um::oleauto::SafeArrayAccessData;
use winapi::um::oleauto::SafeArrayUnaccessData;
use widestring::WideCStr;
use failure::Error;
use winapi::shared::wtypes::BSTR;
use winapi::shared::minwindef::UINT;
use winapi::shared::ntdef::LONG;
use winapi::shared::winerror::HRESULT;
use winapi::um::oaidl::{SAFEARRAY};


extern "system" {
    pub fn SafeArrayGetLBound(psa: *mut SAFEARRAY, nDim: UINT, plLbound: *mut LONG) -> HRESULT;

    pub fn SafeArrayGetUBound(psa: *mut SAFEARRAY, nDim: UINT, plUbound: *mut LONG) -> HRESULT;

    pub fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
}


pub fn get_string_array(arr: *mut SAFEARRAY) -> Result<Vec<String>, Error> {
    let mut p_data = NULL;
    let mut lstart: i32 = 0;
    let mut lend: i32 = 0;

    unsafe {
        check_hres(SafeArrayGetLBound(arr, 1, &mut lstart as _))?;
        check_hres(SafeArrayGetUBound(arr, 1, &mut lend as _))?;
        check_hres(SafeArrayAccessData(arr, &mut p_data))?;
    }

    // We have no data, return an empty vec.
    if lend == -1 {
        return Ok(vec![]);
    }

    let mut p_data: *mut BSTR = p_data as _;

    let mut data_slice = unsafe { slice::from_raw_parts(p_data, lend as usize + 1) };

    let mut props = vec![];

    for prop_name_bstr in data_slice[(lstart as usize)..].iter() {
        let prop_name: &WideCStr = unsafe { WideCStr::from_ptr_str(*prop_name_bstr) };

        props.push(prop_name.to_string()?)
    }

    unsafe {
        check_hres(SafeArrayUnaccessData(arr))?;
        check_hres(SafeArrayDestroy(arr))?;
    }

    Ok(props)
}