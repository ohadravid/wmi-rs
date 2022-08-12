use crate::utils::{check_hres, WMIError};
use crate::Variant;
use std::iter::Iterator;
use std::slice;
use widestring::WideCStr;
use winapi::{
    shared::wtypes::*,
    shared::{ntdef::NULL, wtypes::BSTR},
    um::{
        oaidl::SAFEARRAY,
        oleauto::{
            SafeArrayAccessData, SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
        },
    },
};

#[derive(Debug)]
pub struct SafeArrayAccessor<T> {
    arr: *mut SAFEARRAY,
    p_data: *mut T,
    lower_bound: i32,
    upper_bound: i32,
}

/// An accessor to SafeArray, which:
/// 1. Locks the array so the data can be read.
/// 2. Unlocks the array once dropped.
///
/// Pointers to a Safe Array can come from different places (like GetNames, WMI property value),
/// which can have different drop behavior (GetNames require the caller to deallocate the array,
/// while a WMI property must be deallocated via VariantClear).
///
/// For this reason, we don't have a `struct SafeArray`.
///
/// However, accessing the data of the array must be done using a lock, which is the responsibility
/// of this struct.
///
impl<T> SafeArrayAccessor<T> {
    /// Creates a new Accessor, locking the given array,
    ///
    /// # Safety
    ///
    /// This function is unsafe as it is the caller's responsibility to verify that the array is
    /// of items of type T.
    pub unsafe fn new(arr: *mut SAFEARRAY) -> Result<Self, WMIError> {
        let mut p_data = NULL;
        let mut lower_bound: i32 = 0;
        let mut upper_bound: i32 = 0;

        unsafe {
            check_hres(SafeArrayGetLBound(arr, 1, &mut lower_bound as _))?;
            check_hres(SafeArrayGetUBound(arr, 1, &mut upper_bound as _))?;
            check_hres(SafeArrayAccessData(arr, &mut p_data))?;
        }

        Ok(Self {
            arr,
            p_data: p_data as *mut T,
            lower_bound,
            upper_bound,
        })
    }

    /// Return a slice which can access the data of the array.
    pub fn as_slice(&self) -> &[T] {
        // upper_bound can be -1, in which case the array is empty and we will return a 0 length slice.
        let data_slice =
            unsafe { slice::from_raw_parts(self.p_data, (self.upper_bound + 1) as usize) };

        &data_slice[(self.lower_bound as usize)..]
    }
}

impl<T> Drop for SafeArrayAccessor<T> {
    fn drop(&mut self) {
        // TOOD: Should we handle errors in some way?
        unsafe {
            let _result = check_hres(SafeArrayUnaccessData(self.arr));
        }
    }
}

/// # Safety
///
/// The caller must ensure that the array is valid and contains only strings.
pub unsafe fn safe_array_to_vec_of_strings(arr: *mut SAFEARRAY) -> Result<Vec<String>, WMIError> {
    let items = safe_array_to_vec(arr, VT_BSTR)?;

    let string_items = items
        .into_iter()
        .map(|item| match item {
            Variant::String(s) => s,
            _ => unreachable!(),
        })
        .collect();

    Ok(string_items)
}

/// # Safety
///
/// The caller must ensure that the array is valid.
pub unsafe fn safe_array_to_vec(
    arr: *mut SAFEARRAY,
    item_type: u32,
) -> Result<Vec<Variant>, WMIError> {
    fn copy_type_to_vec<T, F>(
        arr: *mut SAFEARRAY,
        variant_builder: F,
    ) -> Result<Vec<Variant>, WMIError>
    where
        T: Copy,
        F: Fn(T) -> Variant,
    {
        let mut items = vec![];

        let accessor = unsafe { SafeArrayAccessor::<T>::new(arr)? };

        for item in accessor.as_slice().iter() {
            items.push(variant_builder(*item));
        }

        Ok(items)
    }

    match item_type {
        VT_I1 => copy_type_to_vec(arr, Variant::I1),
        VT_I2 => copy_type_to_vec(arr, Variant::I2),
        VT_I4 => copy_type_to_vec(arr, Variant::I4),
        VT_I8 => copy_type_to_vec(arr, Variant::I8),
        VT_UI1 => copy_type_to_vec(arr, Variant::UI1),
        VT_UI2 => copy_type_to_vec(arr, Variant::UI2),
        VT_UI4 => copy_type_to_vec(arr, Variant::UI4),
        VT_UI8 => copy_type_to_vec(arr, Variant::UI8),
        VT_R4 => copy_type_to_vec(arr, Variant::R4),
        VT_R8 => copy_type_to_vec(arr, Variant::R8),
        VT_BSTR => {
            let mut items = vec![];
            let accessor = unsafe { SafeArrayAccessor::<BSTR>::new(arr)? };

            for item_bstr in accessor.as_slice().iter() {
                let item: &WideCStr = unsafe { WideCStr::from_ptr_str(*item_bstr) };

                items.push(Variant::String(item.to_string()?));
            }
            Ok(items)
        }
        // TODO: Add support for all other types of arrays.
        _ => Err(WMIError::UnimplementedArrayItem),
    }
}
