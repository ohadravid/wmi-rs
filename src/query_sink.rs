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
use log::{trace, warn};
use std::ptr::NonNull;
use wio::com::ComPtr;
use async_channel::Sender;
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
    sender: Sender<Vec<IWbemClassWrapper>>,
}

impl QuerySink {
    pub fn new(sender: Sender<Vec<IWbemClassWrapper>>) -> ComPtr<IWbemObjectSink> {
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
        // apObjArray The array memory itself is read-only, and is owned by the caller of the method.
        // Call AddRef on each element of apObjArray to borrow.
        // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-indicate

        // TODO: Document when ObjectCount is <=0
        if lObjectCount <= 0 {
            return WBEM_NO_ERROR as i32;
        }

        let lObjectCount = lObjectCount as usize;
        let tx = self.sender.clone();
        let mut result = Vec::<IWbemClassWrapper>::with_capacity(lObjectCount);

        unsafe {
            // TODO: check if pointers are non null
            // Iterate over input array to extract ClassObjects
            for i in 0..lObjectCount {
                let p_el = *apObjArray.offset(i as isize);
                let wbemClassObject = IWbemClassWrapper::new(NonNull::new(p_el));
                // call AddRef because object will be held after the end of Indicate
                wbemClassObject.add_ref();
                result.push(wbemClassObject);
            }
        }

        // send the result to the receiver
        if let Err(e) = tx.try_send(result) {
            warn!("Error while sending result: {}", e);
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
    async fn it_works_async() {
        let con = wmi_con();
        let (tx, rx) = async_channel::unbounded();
        let p_sink: ComPtr<IWbemObjectSink> = QuerySink::new(tx);

        let raw_os = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#).unwrap();
        let raw_os2 = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#).unwrap();
        let ptr: *mut IWbemClassObject = raw_os.inner.unwrap().as_ptr();
        let ptr2: *mut IWbemClassObject = raw_os2.inner.unwrap().as_ptr();

        let mut arr = vec![ptr, ptr2];

        assert_eq!(rx.len(), 0);

        unsafe {p_sink.Indicate(arr.len() as i32, arr.as_mut_ptr());}

        assert_eq!(rx.len(), 1);

        println!("Number of senders: {}", rx.sender_count());
        println!("Number of receivers: {}", rx.receiver_count());
        let result: Vec::<IWbemClassWrapper> = rx.recv().await.unwrap();
        
        assert_eq!(result.len(), 2);

        for obj in &result {
            assert_eq!(obj.class().unwrap().as_str(), "Win32_OperatingSystem");
        }

    }
}
