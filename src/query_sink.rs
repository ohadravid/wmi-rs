use winapi::{
    um::wbemcli::{
        {IWbemClassObject,IWbemObjectSink, IWbemObjectSinkVtbl},
        WBEM_NO_ERROR,
    },
    shared::{
        ntdef::HRESULT,
        wtypes::BSTR,
        winerror::E_POINTER,
    },
    ctypes::{
        c_long,
    },
};
use com_impl::{ComImpl, VTable, Refcount};
use log::{trace, warn};
use std::ptr::NonNull;
use wio::com::ComPtr;
use async_channel::Sender;
use crate::result_enumerator::IWbemClassWrapper;
use crate::WMIError;

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
    sender: Sender<Result<IWbemClassWrapper, WMIError>>,
}

impl QuerySink {
    pub fn new(sender: Sender<Result<IWbemClassWrapper, WMIError>>) -> ComPtr<IWbemObjectSink> {
        let ptr = QuerySink::create_raw(sender);
        let ptr = ptr as *mut IWbemObjectSink;
        unsafe { ComPtr::from_raw(ptr) }
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
        // TODO: Document when ObjectCount is <=0
        if lObjectCount <= 0 {
            return WBEM_NO_ERROR as i32;
        }

        let lObjectCount = lObjectCount as usize;
        let tx = self.sender.clone();

        unsafe {
            // The array memory of apObjArray is read-only, and is owned by the caller of the Indicate method.
            // The call to AddRef on each element of apObjArray to borrow is done by the IWbemClassWrapper::clone
            // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-indicate
            for i in 0..lObjectCount {
                // iterate over apObjArray elements
                let p_el = *apObjArray.offset(i as isize);
                // check for null pointer before cloning
                if p_el.is_null() {
                    // TODO: check how Indicate error code are handled by WMI
                    // TODO: inform receiver with tx.try_send(Err(...))
                    // See https://docs.microsoft.com/en-us/windows/win32/learnwin32/error-handling-in-com
                    return E_POINTER;
                }
                // extend ClassObject lifespan beyond scope of Indicate method
                let wbemClassObject = IWbemClassWrapper::clone(NonNull::new(p_el));
                // send the result to the receiver
                if let Err(e) = tx.try_send(Ok(wbemClassObject)) {
                    // TODO: send error back to WMI
                    warn!("Error while sending object: {}", e);
                }
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


#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::*;

    #[async_std::test]
    async fn it_should_use_async_channel_to_send_result() {
        let con = wmi_con();
        let (tx, rx) = async_channel::unbounded();
        let p_sink: ComPtr<IWbemObjectSink> = QuerySink::new(tx);

        let raw_os = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#).unwrap();
        let raw_os2 = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#).unwrap();
        let ptr: *mut IWbemClassObject = raw_os.inner.unwrap().as_ptr();
        let ptr2: *mut IWbemClassObject = raw_os2.inner.unwrap().as_ptr();

        let mut arr = vec![ptr, ptr2];

        assert_eq!(rx.len(), 0);

        // tests on ref count before Indicate call
        unsafe {
            let test_ptr = &ptr;
            let refcount = test_ptr.as_ref().unwrap().AddRef();
            assert_eq!(refcount, 2);
            let refcount = test_ptr.as_ref().unwrap().Release();
            assert_eq!(refcount, 1);
        }

        unsafe {p_sink.Indicate(arr.len() as i32, arr.as_mut_ptr());}
        // tests on ref count after Indicate call
        unsafe {
            let test_ptr = &ptr;
            let refcount = test_ptr.as_ref().unwrap().AddRef();
            assert_eq!(refcount, 3);
            let refcount = test_ptr.as_ref().unwrap().Release();
            assert_eq!(refcount, 2);
        }

        assert_eq!(rx.len(), 2);

        assert_eq!(rx.sender_count(), 1);
        assert_eq!(rx.receiver_count(), 1);

        if let Ok(first) = rx.recv().await.unwrap() {
            assert_eq!(first.class().unwrap().as_str(), "Win32_OperatingSystem");
        } else {
            assert!(false);
        }
        
        assert_eq!(rx.len(), 1);

        if let Ok(second) = rx.recv().await.unwrap() {
            assert_eq!(second.class().unwrap().as_str(), "Win32_OperatingSystem");
        } else {
            assert!(false);
        }

        assert_eq!(rx.len(), 0);
    }
}
