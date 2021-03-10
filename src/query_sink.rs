use winapi::{
    um::wbemcli::{
        {IWbemClassObject},
        WBEM_S_NO_ERROR,
        WBEM_STATUS_COMPLETE,
    },
    shared::{
        ntdef::HRESULT,
        wtypes::BSTR,
        winerror::{E_POINTER, E_FAIL},
    },
    ctypes::{
        c_long,
    },
};
use log::{trace, warn};
use std::ptr::NonNull;
use async_channel::Sender;
use crate::result_enumerator::IWbemClassWrapper;
use crate::WMIError;

com::interfaces! {
    #[uuid("7C857801-7381-11CF-884D-00AA004B2E24")]
    pub unsafe interface IWbemObjectSink: com::interfaces::IUnknown {
        unsafe fn indicate(
            &self,
            lObjectCount: c_long,
            apObjArray: *mut *mut IWbemClassObject
        ) -> HRESULT;

        unsafe fn set_status(
            &self,
            lFlags: c_long,
            _hResult: HRESULT,
            _strParam: BSTR,
            _pObjParam: *mut IWbemClassObject
        ) -> HRESULT;
    }
}


com::class! {
    // Option is required because `Default` is required by the `class!` macro.
    pub class QuerySink: IWbemObjectSink {
        sender: Option<Sender<Result<IWbemClassWrapper, WMIError>>>,
    }
    
    /// Implementation for [IWbemObjectSink](https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nn-wbemcli-iwbemobjectsink).
    /// This [Sink](https://en.wikipedia.org/wiki/Sink_(computing)) 
    /// receives asynchronously the result of the query, through Indicate calls. 
    /// When finished,the SetStatus method is called.
    /// # <https://docs.microsoft.com/fr-fr/windows/win32/wmisdk/example--getting-wmi-data-from-the-local-computer-asynchronously>
    impl IWbemObjectSink for QuerySink {
        unsafe fn indicate(
            &self,
            lObjectCount: c_long,
            apObjArray: *mut *mut IWbemClassObject
        ) -> HRESULT {
            trace!("Indicate call with {} objects", lObjectCount);
            // Case of an incorrect or too restrictive query
            if lObjectCount <= 0 {
                return WBEM_S_NO_ERROR as i32;
            }
    
            let lObjectCount = lObjectCount as usize;
            let tx = self.sender.as_ref().unwrap().clone();
    
            unsafe {
                // The array memory of apObjArray is read-only 
                // and is owned by the caller of the Indicate method.
                // IWbemClassWrapper::clone calls AddRef on each element 
                // of apObjArray to make sure that they are not released, 
                // according to COM rules.
                // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-indicate
                // For error codes, see https://docs.microsoft.com/en-us/windows/win32/learnwin32/error-handling-in-com
                for i in 0..lObjectCount {
                    if let Some(p_el) = NonNull::new(*apObjArray.offset(i as isize)) {
                        let wbemClassObject = IWbemClassWrapper::clone(p_el);
    
                        if let Err(e) = tx.try_send(Ok(wbemClassObject)) {
                            warn!("Error while sending object through async_channel: {}", e);
                            return E_FAIL;
                        }
                    } else {
                        if let Err(e) = tx.try_send(Err(WMIError::NullPointerResult)) {
                            warn!("Error while sending error code through async_channel: {}", e);
                        }
                        return E_POINTER;
                    }
                    
                }
            }
    
            WBEM_S_NO_ERROR as i32
        }
    
        unsafe fn set_status(
            &self,
            lFlags: c_long,
            _hResult: HRESULT,
            _strParam: BSTR,
            _pObjParam: *mut IWbemClassObject
        ) -> HRESULT {
            // SetStatus is called only once as flag=WBEM_FLAG_BIDIRECTIONAL in ExecQueryAsync
            // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-setstatus
            // If you do not specify WBEM_FLAG_SEND_STATUS when calling your provider or service method,
            // you are guaranteed to receive one and only one call to SetStatus
    
            if lFlags == WBEM_STATUS_COMPLETE as i32 {
                trace!("End of async result, closing transmitter");
                self.sender.as_ref().unwrap().close();
            }
            WBEM_S_NO_ERROR as i32
        }
    }
}


#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::*;
    use winapi::shared::ntdef::NULL;

    #[async_std::test]
    async fn _async_it_should_use_async_channel_to_send_result() {
        let con = wmi_con();
        let (tx, rx) = async_channel::unbounded();
        let sink = QuerySink::allocate(Some(tx));
        let p_sink = sink.query_interface::<IWbemObjectSink>().unwrap();

        let raw_os = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#).unwrap();
        let raw_os2 = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#).unwrap();
        let ptr: *mut IWbemClassObject = raw_os.inner.as_ptr();
        let ptr2: *mut IWbemClassObject = raw_os2.inner.as_ptr();

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

        unsafe {p_sink.indicate(arr.len() as i32, arr.as_mut_ptr());}
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

    #[test]
    fn _async_it_should_close_async_channel_after_set_status_call() {
        let (tx, rx) = async_channel::unbounded();
        let sink = QuerySink::allocate(Some(tx));
        let p_sink = sink.query_interface::<IWbemObjectSink>().unwrap();

        assert!(!rx.is_closed());

        unsafe {p_sink.set_status(WBEM_STATUS_COMPLETE as i32, 0, NULL as BSTR, NULL as *mut IWbemClassObject);}

        assert!(rx.is_closed());
    }

    #[async_std::test]
    async fn _async_it_should_return_e_pointer_after_indicate_call_with_null_pointer() {
        let (tx, rx) = async_channel::unbounded();
        let sink = QuerySink::allocate(Some(tx));
        let p_sink = sink.query_interface::<IWbemObjectSink>().unwrap();

        let mut arr = vec![NULL as *mut IWbemClassObject];
        let result;

        unsafe { result = p_sink.indicate(1, arr.as_mut_ptr()) }

        match rx.recv().await.unwrap() {
            Err(WMIError::NullPointerResult) => assert!(true),
            _ => assert!(false),
        }

        assert_eq!(result, E_POINTER);
    }
}
