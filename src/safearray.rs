use crate::utils::check_hres;
use crate::Variant;
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

// TODO: This should be part of winapi-rs.
extern "system" {
    pub fn SafeArrayGetLBound(psa: *mut SAFEARRAY, nDim: UINT, plLbound: *mut LONG) -> HRESULT;

    pub fn SafeArrayGetUBound(psa: *mut SAFEARRAY, nDim: UINT, plUbound: *mut LONG) -> HRESULT;

    pub fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
}

pub fn get_string_array(arr: *mut SAFEARRAY) -> Result<Vec<String>, Error> {
    let mut p_data = NULL;

    let mut lower_bound: i32 = 0;
    let mut upper_bound: i32 = 0;

    unsafe {
        check_hres(SafeArrayGetLBound(arr, 1, &mut lower_bound as _))?;
        check_hres(SafeArrayGetUBound(arr, 1, &mut upper_bound as _))?;
        check_hres(SafeArrayAccessData(arr, &mut p_data))?;
    }

    // We know that we expect an array of BSTRs.
    let p_data: *mut BSTR = p_data as _;

    // upper_bound can be -1, in which case the array is empty and we will do nothing.
    let data_slice = unsafe { slice::from_raw_parts(p_data, (upper_bound + 1) as usize) };

    let mut items = vec![];

    for item_bstr in data_slice[(lower_bound as usize)..].iter() {
        let item: &WideCStr = unsafe { WideCStr::from_ptr_str(*item_bstr) };

        items.push(item.to_string()?)
    }

    // TODO: Make sure this happens even on errors.
    unsafe {
        check_hres(SafeArrayUnaccessData(arr))?;
    }

    Ok(items)
}

pub fn safe_array_to_vec(arr: *mut SAFEARRAY, item_type: u32) -> Result<Vec<Variant>, Error> {
    let mut p_data = NULL;

    let mut lower_bound: i32 = 0;
    let mut upper_bound: i32 = 0;

    unsafe {
        check_hres(SafeArrayGetLBound(arr, 1, &mut lower_bound as _))?;
        check_hres(SafeArrayGetUBound(arr, 1, &mut upper_bound as _))?;
        check_hres(SafeArrayAccessData(arr, &mut p_data))?;
    }

    let mut items = vec![];

    match item_type {
        VT_I4 => {
            unimplemented!();

            // We know that we expect an array of this type.
            let p_data: *mut i32 = p_data as _;

            // upper_bound can be -1, in which case the array is empty and we will do nothing.
            let data_slice = unsafe { slice::from_raw_parts(p_data, (upper_bound + 1) as usize) };

            for item in data_slice[(lower_bound as usize)..].iter() {
                items.push(Variant::I4(*item))
            }
        }

        // TODO: Add support for all other types of arrays.
        _ => unimplemented!(),
    }

    // TODO: Make sure this happens even on errors.
    unsafe {
        check_hres(SafeArrayUnaccessData(arr))?;
    }

    Ok(items)
}
