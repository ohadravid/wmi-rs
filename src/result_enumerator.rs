use crate::de::wbem_class_de::from_wbem_class_obj;
use crate::{
    BStr,
    connection::WMIConnection, safearray::safe_array_to_vec_of_strings, utils::check_hres, Variant,
    WMIError,
};
use log::trace;
use serde::de;
use std::convert::TryInto;
use std::{mem, ptr, ptr::NonNull};
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

/// A wrapper around a raw pointer to IWbemClassObject, which also takes care of releasing
/// the object when dropped.
///
#[derive(Debug)]
pub struct IWbemClassWrapper {
    pub inner: Option<NonNull<IWbemClassObject>>,
}

impl IWbemClassWrapper {
    pub unsafe fn new(ptr: Option<NonNull<IWbemClassObject>>) -> Self {
        Self { inner: ptr }
    }

    /// Creates a copy of the pointer and calls [AddRef] to increment Reference Count
    /// See [Managing the lifetime of an object] in the documentation
    /// [Managing the lifetime of an object]: https://docs.microsoft.com/en-us/windows/win32/learnwin32/managing-the-lifetime-of-an-object
    /// [AddRef]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-addref
    ///
    pub unsafe fn clone(ptr: NonNull<IWbemClassObject>) -> Self {
        let refcount = ptr.as_ref().AddRef();
        trace!("Reference count: {}", refcount);
        Self::new(Some(ptr))
    }

    /// Return the names of all the properties of the given object.
    ///
    pub fn list_properties(&self) -> Result<Vec<String>, WMIError> {
        // This will store the properties names from the GetNames call.
        let mut p_names = NULL as *mut SAFEARRAY;

        let ptr = self.inner.unwrap().as_ptr();

        unsafe {
            check_hres((*ptr).GetNames(
                ptr::null(),
                (WBEM_FLAG_ALWAYS | WBEM_FLAG_NONSYSTEM_ONLY) as i32,
                ptr::null_mut(),
                &mut p_names,
            ))?;
        

            let res = safe_array_to_vec_of_strings(p_names);

            check_hres(SafeArrayDestroy(p_names))?;

            res
        }
    }

    pub fn get_property(&self, property_name: &str) -> Result<Variant, WMIError> {
        let name_prop = BStr::from_str(property_name)?;

        let mut vt_prop: VARIANT = unsafe { mem::zeroed() };

        unsafe {
            (*self.inner.unwrap().as_ptr()).Get(
                name_prop.as_lpcwstr(),
                0,
                &mut vt_prop,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        

            let property_value = Variant::from_variant(vt_prop)?;

            VariantClear(&mut vt_prop);

            Ok(property_value)
        }
    }

    pub fn path(&self) -> Result<String, WMIError> {
        self.get_property("__Path").and_then(Variant::try_into)
    }

    pub fn class(&self) -> Result<String, WMIError> {
        self.get_property("__Class").and_then(Variant::try_into)
    }

    pub fn into_desr<T>(self) -> Result<T, WMIError>
    where
        T: de::DeserializeOwned,
    {
        from_wbem_class_obj(&self).map_err(WMIError::from)
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
    _wmi_con: &'a WMIConnection,
    p_enumerator: Option<NonNull<IEnumWbemClassObject>>,
}

impl<'a> QueryResultEnumerator<'a> {
    pub unsafe fn new(wmi_con: &'a WMIConnection, p_enumerator: *mut IEnumWbemClassObject) -> Self {
        Self {
            _wmi_con: wmi_con,
            p_enumerator: NonNull::new(p_enumerator),
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
    type Item = Result<IWbemClassWrapper, WMIError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pcls_obj = NULL as *mut IWbemClassObject;
        let mut return_value = 0;
        
        let raw_enumerator_prt = self.p_enumerator?.as_ptr();

        let res = unsafe {
            check_hres((*raw_enumerator_prt).Next(
                WBEM_INFINITE as i32,
                1,
                &mut pcls_obj,
                &mut return_value,
            ))
        };

        if let Err(e) = res {
            return Some(Err(e));
        }

        if return_value == 0 {
            return None;
        }

        trace!(
            "Got enumerator {:?} and obj {:?}",
            self.p_enumerator,
            pcls_obj
        );

        let pcls_wrapper = unsafe { IWbemClassWrapper::new(NonNull::new(pcls_obj)) };

        Some(Ok(pcls_wrapper))
    }
}
