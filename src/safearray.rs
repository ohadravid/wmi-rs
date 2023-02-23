use crate::{utils::{WMIError, WMIResult}, Variant};
use std::{iter::Iterator, slice, ptr::null_mut};
use windows::Win32::System::Com::{self, SAFEARRAY, VT_BSTR, VARENUM};
use windows::Win32::System::Ole::{SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayAccessData, SafeArrayUnaccessData};
use windows::core::BSTR;

#[derive(Debug)]
pub struct SafeArrayAccessor<'a, T> {
    arr: &'a SAFEARRAY,
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
impl<'a, T> SafeArrayAccessor<'a, T> {
    /// Creates a new Accessor, locking the given array,
    ///
    /// # Safety
    ///
    /// This function is unsafe as it is the caller's responsibility to verify that the array is
    /// of items of type T.
    pub fn new(arr: &'a SAFEARRAY) -> WMIResult<Self> {
        let mut p_data = null_mut();

        let lower_bound = unsafe { SafeArrayGetLBound(arr, 1)? };
        let upper_bound = unsafe { SafeArrayGetUBound(arr, 1)? };
        unsafe { SafeArrayAccessData(arr, &mut p_data)? };

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

impl<'a, T> Drop for SafeArrayAccessor<'a, T> {
    fn drop(&mut self) {
        // TOOD: Should we handle errors in some way?
        unsafe {
            let _result = SafeArrayUnaccessData(self.arr);
        }
    }
}

/// # Safety
///
/// The caller must ensure that the array is valid and contains only strings.
pub fn safe_array_to_vec_of_strings(arr: &SAFEARRAY) -> WMIResult<Vec<String>> {
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
pub fn safe_array_to_vec(
    arr: &SAFEARRAY,
    item_type: VARENUM,
) -> WMIResult<Vec<Variant>> {
    fn copy_type_to_vec<T, F>(
        arr: &SAFEARRAY,
        variant_builder: F,
    ) -> WMIResult<Vec<Variant>>
    where
        T: Copy,
        F: Fn(T) -> Variant,
    {
        let mut items = vec![];

        let accessor = SafeArrayAccessor::<T>::new(arr)?;

        for item in accessor.as_slice().iter() {
            items.push(variant_builder(*item));
        }

        Ok(items)
    }

    match item_type {
        Com::VT_I1 => copy_type_to_vec(arr, Variant::I1),
        Com::VT_I2 => copy_type_to_vec(arr, Variant::I2),
        Com::VT_I4 => copy_type_to_vec(arr, Variant::I4),
        Com::VT_I8 => copy_type_to_vec(arr, Variant::I8),
        Com::VT_UI1 => copy_type_to_vec(arr, Variant::UI1),
        Com::VT_UI2 => copy_type_to_vec(arr, Variant::UI2),
        Com::VT_UI4 => copy_type_to_vec(arr, Variant::UI4),
        Com::VT_UI8 => copy_type_to_vec(arr, Variant::UI8),
        Com::VT_R4 => copy_type_to_vec(arr, Variant::R4),
        Com::VT_R8 => copy_type_to_vec(arr, Variant::R8),
        Com::VT_BSTR => {
            let mut items = vec![];
            let accessor = unsafe { SafeArrayAccessor::<BSTR>::new(arr)? };

            for item_bstr in accessor.as_slice().iter() {
                items.push(Variant::String(item_bstr.try_into()?));
            }
            Ok(items)
        }
        // TODO: Add support for all other types of arrays.
        _ => Err(WMIError::UnimplementedArrayItem),
    }
}
