use crate::{
    connection::WMIConnection,
    de::wbem_class_de::from_wbem_class_obj,
    safearray::safe_array_to_vec_of_strings,
    utils::check_hres,
    WMIResult,
    WMIError,
    Variant,
    BStr,
};
use winapi::{
    shared::ntdef::NULL,
    um::{
        oaidl::{SAFEARRAY, VARIANT},
        wbemcli::{IEnumWbemClassObject, IWbemClassObject, WBEM_FLAG_ALWAYS, WBEM_FLAG_NONSYSTEM_ONLY, WBEM_INFINITE},
        oleauto::{SafeArrayDestroy, VariantClear},
    },
};
use std::{
    ptr::{self, NonNull},
    convert::TryInto,
    cell::RefCell,
    mem,
};
use serde::{ser::{SerializeStruct, Error}, de, Serialize};
use log::trace;

/// A wrapper around a raw pointer to IWbemClassObject, which also takes care of releasing
/// the object when dropped.
///
#[derive(Debug, PartialEq, Eq)]
pub struct IWbemClassWrapper {
    pub inner: NonNull<IWbemClassObject>,
    data: RefCell<Option<WbemClassSerializationData>>,
}

impl IWbemClassWrapper {
    pub unsafe fn new(ptr: NonNull<IWbemClassObject>) -> Self {
        Self { inner: ptr, data: RefCell::default(), }
    }

    /// Creates a copy of the pointer and calls
    /// [AddRef](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-addref)
    /// to increment Reference Count.
    ///
    /// # Safety
    /// See [Managing the lifetime of an object](https://docs.microsoft.com/en-us/windows/win32/learnwin32/managing-the-lifetime-of-an-object)
    /// and [Rules for managing Ref count](https://docs.microsoft.com/en-us/windows/win32/com/rules-for-managing-reference-counts)
    ///
    pub unsafe fn clone(ptr: NonNull<IWbemClassObject>) -> Self {
        let refcount = ptr.as_ref().AddRef();
        trace!("Reference count: {}", refcount);
        Self::new(ptr)
    }

    /// Return the names of all the properties of the given object.
    ///
    pub fn list_properties(&self) -> WMIResult<Vec<String>> {
        // This will store the properties names from the GetNames call.
        let mut p_names = NULL as *mut SAFEARRAY;

        let ptr = self.inner.as_ptr();

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

    pub fn get_property(&self, property_name: &str) -> WMIResult<Variant> {
        let name_prop = BStr::from_str(property_name)?;

        let mut vt_prop: VARIANT = unsafe { mem::zeroed() };

        let mut cim_type = 0;

        unsafe {
            check_hres((*self.inner.as_ptr()).Get(
                name_prop.as_lpcwstr(),
                0,
                &mut vt_prop,
                &mut cim_type,
                ptr::null_mut(),
            ))?;

            let property_value =
                Variant::from_variant(vt_prop)?.convert_into_cim_type(cim_type as _)?;

            check_hres(VariantClear(&mut vt_prop))?;

            Ok(property_value)
        }
    }

    pub fn path(&self) -> WMIResult<String> {
        self.get_property("__Path").and_then(Variant::try_into)
    }

    pub fn class(&self) -> WMIResult<String> {
        self.get_property("__Class").and_then(Variant::try_into)
    }

    pub fn into_desr<T>(self) -> WMIResult<T>
    where
        T: de::DeserializeOwned,
    {
        from_wbem_class_obj(self).map_err(WMIError::from)
    }

    fn serialization_data(&self) -> WMIResult<&'static WbemClassSerializationData> {
        if self.data.borrow().is_none() {
            let name = self.class()?;
            let props = self.list_properties()?;
            *(self.data.borrow_mut()) = Some(WbemClassSerializationData { name, properties: props });
        }
        Ok(unsafe { Box::leak(Box::from_raw(self.data.borrow_mut().as_mut().unwrap() as *mut _)) })
    }
}

impl Clone for IWbemClassWrapper {
    fn clone(&self) -> Self {
        unsafe { Self::clone(self.inner) }
    }
}

impl Drop for IWbemClassWrapper {
    fn drop(&mut self) {
        let ptr = self.inner.as_ptr();
        unsafe {
            (*ptr).Release();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WbemClassSerializationData {
    name: String,
    properties: Vec<String>,
}

impl Serialize for IWbemClassWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let data = self.serialization_data().map_err(Error::custom)?;
        let mut s = serializer.serialize_struct(&data.name, data.properties.len())?;
        for property in data.properties.iter() {
            let value = self.get_property(&property).unwrap();
            s.serialize_field(property, &value)?;
        }
        s.end()
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
    type Item = WMIResult<IWbemClassWrapper>;

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

        let pcls_ptr = NonNull::new(pcls_obj).ok_or(WMIError::NullPointerResult);

        match pcls_ptr {
            Err(e) => Some(Err(e)),
            Ok(pcls_ptr) => Some(Ok(unsafe { IWbemClassWrapper::new(pcls_ptr) })),
        }
    }
}
