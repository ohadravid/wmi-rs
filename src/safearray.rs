use crate::utils::check_hres;
use failure::Error;
use std::slice;
use widestring::WideCStr;
use winapi::{
    shared::{
        minwindef::UINT,
        ntdef::{LONG, NULL},
        winerror::HRESULT,
        wtypes::BSTR,
    },
    um::{oaidl::SAFEARRAY, oleauto::SafeArrayAccessData, oleauto::SafeArrayUnaccessData},
};

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

    let p_data: *mut BSTR = p_data as _;

    // lend can be -1, in which case the array is empty and we will do nothing.
    let data_slice = unsafe { slice::from_raw_parts(p_data, (lend + 1) as usize) };

    let mut props = vec![];

    for prop_name_bstr in data_slice[(lstart as usize)..].iter() {
        let prop_name: &WideCStr = unsafe { WideCStr::from_ptr_str(*prop_name_bstr) };

        props.push(prop_name.to_string()?)
    }

    // TODO: Make sure this happens even on errors.
    unsafe {
        check_hres(SafeArrayUnaccessData(arr))?;
    }

    Ok(props)
}
