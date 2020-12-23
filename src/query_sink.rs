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
        _lObjectCount: c_long,
        _apObjArray: *mut *mut IWbemClassObject
    ) -> HRESULT {
        println!("Indicate was called");

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
