use winapi::{
    um::wbemcli::{
        IWbemClassObject,
        WBEM_NO_ERROR,
    },
    shared::{
        ntdef::HRESULT,
        wtypes::BSTR,
    },
    ctypes::{
        c_long,
    },
};
use winapi::um::wbemcli::{IWbemObjectSink, IWbemObjectSinkVtbl};
use com_impl::{ComImpl, VTable, Refcount};
use log::trace;
use std::ptr::NonNull;
use crate::result_enumerator::IWbemClassWrapper;

/// Implementation for IWbemObjectSink.
/// This [Sink] receives asynchronously the result of the query,
/// through Indicate calls. When finished,the SetStatus method
/// is called.
/// [Sink]: https://en.wikipedia.org/wiki/Sink_(computing)
/// # https://docs.microsoft.com/fr-fr/windows/win32/wmisdk/example--getting-wmi-data-from-the-local-computer-asynchronously
#[repr(C)]
#[derive(ComImpl)]
#[interfaces(IWbemObjectSink)]
pub struct QuerySink {
    vtbl: VTable<IWbemObjectSinkVtbl>,
    refcount: Refcount,
}

impl QuerySink {
    pub fn new_ptr() -> *mut IWbemObjectSink {
        let ptr = QuerySink::create_raw();
        ptr as *mut IWbemObjectSink
    }
}


#[com_impl::com_impl]
unsafe impl IWbemObjectSink for QuerySink {
    pub unsafe fn indicate(
        &self,
        lObjectCount: c_long,
        apObjArray: *mut *mut IWbemClassObject
    ) -> HRESULT {
        trace!("Indicate call with {} objects", lObjectCount);
        // apObjArray The array memory itself is read-only, and is owned by the caller of the method.
        // Call AddRef on each element of apObjArray to borrow.
        // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-indicate

        // TODO: Document when ObjectCount is <=0
        if lObjectCount <= 0 {
            return WBEM_NO_ERROR as i32;
        }

        unsafe {
            // TODO: check if pointers are non null
            // Iterate over result array to extract ClassObjects
            for i in 0..lObjectCount {
                let p_el = *apObjArray.offset(i as isize);
                let wbemClassObject = IWbemClassWrapper::new(NonNull::new(p_el));
                // call to AddRef because object will be held after the end of Indicate
                wbemClassObject.add_ref();
                // TODO: store wbemCLassObject in ThreadSafe Array
                trace!("{:?}", wbemClassObject.list_properties());
            }
        }

        WBEM_NO_ERROR as i32
    }

    pub unsafe fn set_status(
        &self,
        _lFlags: c_long,
        _hResult: HRESULT,
        _strParam: BSTR,
        _pObjParam: *mut IWbemClassObject
    ) -> HRESULT {
        WBEM_NO_ERROR as i32
    }
}
