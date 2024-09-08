use crate::{
    utils::{WMIError, WMIResult},
    Variant,
};
use std::{iter::Iterator, ptr::null_mut};
use windows::core::BSTR;
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{SafeArrayAccessData, SafeArrayUnaccessData};
use windows::Win32::System::Variant::*;

#[derive(Debug)]
pub struct SafeArrayAccessor<'a, T> {
    arr: &'a SAFEARRAY,
    p_data: *mut T,
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
    pub unsafe fn new(arr: &'a SAFEARRAY) -> WMIResult<Self> {
        let mut p_data = null_mut();

        if arr.cDims != 1 {
            return Err(WMIError::UnimplementedArrayItem);
        }

        unsafe { SafeArrayAccessData(arr, &mut p_data)? };

        Ok(Self {
            arr,
            p_data: p_data as *mut T,
        })
    }

    /// Return an iterator over the items of the array.
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        // Safety: We required the caller of `new` to ensure that the array is valid and contains only items of type T (and one dimensional).
        // `SafeArrayAccessData` returns a pointer to the data of the array, which can be accessed for `arr.rgsabound[0].cElements` elements.
        // See: https://learn.microsoft.com/en-us/windows/win32/api/oleauto/nf-oleauto-safearrayaccessdata#examples
        let element_count = self.arr.rgsabound[0].cElements;

        (0..element_count).map(move |i| unsafe { &*self.p_data.offset(i as isize) })
    }
}

impl<'a, T> Drop for SafeArrayAccessor<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let _result = SafeArrayUnaccessData(self.arr);
        }
    }
}

/// # Safety
///
/// The caller must ensure that the array is valid and contains only strings.
pub unsafe fn safe_array_to_vec_of_strings(arr: &SAFEARRAY) -> WMIResult<Vec<String>> {
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
/// The caller must ensure that the array is valid and contains elements on the specified type.
pub unsafe fn safe_array_to_vec(arr: &SAFEARRAY, item_type: VARENUM) -> WMIResult<Vec<Variant>> {
    fn copy_type_to_vec<T, F>(arr: &SAFEARRAY, variant_builder: F) -> WMIResult<Vec<Variant>>
    where
        T: Copy,
        F: Fn(T) -> Variant,
    {
        let accessor = unsafe { SafeArrayAccessor::<T>::new(arr)? };

        Ok(accessor.iter().map(|item| variant_builder(*item)).collect())
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
            let accessor = unsafe { SafeArrayAccessor::<BSTR>::new(arr)? };

            accessor
                .iter()
                .map(|item| item.try_into().map(Variant::String).map_err(WMIError::from))
                .collect()
        }
        // TODO: Add support for all other types of arrays.
        _ => Err(WMIError::UnimplementedArrayItem),
    }
}
