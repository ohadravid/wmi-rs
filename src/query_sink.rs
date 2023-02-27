use crate::{result_enumerator::IWbemClassWrapper, WMIResult, WMIError};
use std::{
    task::{Poll, Waker},
    sync::{Arc, Mutex},
    collections::VecDeque,
    ptr::NonNull,
};
use com::production::ClassAllocation;
use winapi::{
    ctypes::c_long,
    shared::{ntdef::HRESULT, winerror::E_POINTER, wtypes::BSTR},
    um::wbemcli::{IWbemClassObject, WBEM_STATUS_COMPLETE, WBEM_S_NO_ERROR},
};
use futures::Stream;
use log::trace;

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

#[derive(Default)]
pub struct AsyncQueryResultStreamImpl {
    buf: VecDeque<WMIResult<IWbemClassWrapper>>,
    is_done: bool,
    waker: Option<Waker>,
}

/// We wrap the internal objects to ensure that the waker is correctly called when new data is available or when the query is done.
///
/// If the waker is still `None`, we know that `poll_next` has not been called yet.
/// We can fill the buffer, and once `poll_next` is called, it'll return `Poll::Ready` and there's no need to wake the stream manually.
///
/// Once the internal buffer is fully consumed (or empty to begin with) and `poll_next` is called, it'll set the waker and return `Poll::Pending`.
/// Because the waker is set, we can wake the stream.
impl AsyncQueryResultStreamImpl {
    pub fn extend(&mut self, iter: impl IntoIterator<Item = WMIResult<IWbemClassWrapper>>) {
        self.buf.extend(iter);

        if let Some(waker) = self.waker.as_ref() {
            waker.wake_by_ref();
        }
    }

    pub fn set_done(&mut self) {
        self.is_done = true;

        if let Some(waker) = self.waker.as_ref() {
            waker.wake_by_ref();
        }
    }
}

/// A stream of WMI query results.
/// We use a mutex to synchronize the consumer and the calls from the WMI-managed thread.
/// A blocking mutex is used because we want to be runtime agnostic
/// and because according to [`tokio::sync::Mutex`](https://docs.rs/tokio/tokio/tokio/sync/struct.Mutex.html):
/// > The primary use case for the async mutex is to provide shared mutable access to IO resources such as a database connection. If the value behind the mutex is just data, it’s usually appropriate to use a blocking mutex
pub struct AsyncQueryResultStream {
    inner: AsyncQueryResultStreamInner,
    sink: ClassAllocation<QuerySink>,
}

impl AsyncQueryResultStream {
    pub fn new(inner: AsyncQueryResultStreamInner, sink: ClassAllocation<QuerySink>) -> Self {
        Self {
            inner,
            sink,
        }
    }
}

#[derive(Default, Clone)]
pub struct AsyncQueryResultStreamInner(Arc<Mutex<AsyncQueryResultStreamImpl>>);

impl AsyncQueryResultStreamInner {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AsyncQueryResultStreamImpl::default())))
    }

    fn extend(&self, iter: impl IntoIterator<Item = WMIResult<IWbemClassWrapper>>) {
        let mut lock = self.0.lock().unwrap();
        lock.extend(iter);
    }

    fn set_done(&self) {
        let mut lock = self.0.lock().unwrap();
        lock.set_done();
    }
}

impl Stream for AsyncQueryResultStream {
    type Item = WMIResult<IWbemClassWrapper>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let waker = cx.waker();
        let mut inner = self.inner.0.lock().unwrap();

        if !inner
            .waker
            .as_ref()
            .map(|current_waker| waker.will_wake(current_waker))
            .unwrap_or(false)
        {
            inner.waker.replace(waker.clone());
        }

        let next = inner.buf.pop_back();

        match next {
            Some(item) => {
                trace!("poll_next: item found");
                Poll::Ready(Some(item))
            }
            None => {
                if inner.is_done {
                    trace!("poll_next: done");
                    Poll::Ready(None)
                } else {
                    trace!("poll_next: item not found");
                    Poll::Pending
                }
            }
        }
    }
}

com::class! {
    pub class QuerySink: IWbemObjectSink {
        stream: AsyncQueryResultStreamInner,
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
            let mut res = WBEM_S_NO_ERROR as i32;

            // The array memory of apObjArray is read-only
            // and is owned by the caller of the Indicate method.
            // IWbemClassWrapper::clone calls AddRef on each element
            // of apObjArray to make sure that they are not released,
            // according to COM rules.
            // https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemobjectsink-indicate
            // For error codes, see https://docs.microsoft.com/en-us/windows/win32/learnwin32/error-handling-in-com
            self.stream
                .extend((0..lObjectCount).map(|i| {
                if let Some(p_el) = NonNull::new(*apObjArray.add(i)) {
                    let wbemClassObject = unsafe {
                        IWbemClassWrapper::clone(p_el)
                    };

                    Ok(wbemClassObject)
                } else {
                    res = E_POINTER;
                    Err(WMIError::NullPointerResult)
                }
            }));

            res
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
                self.stream.set_done();
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
    use futures::StreamExt;
    use winapi::shared::ntdef::NULL;

    #[async_std::test]
    async fn async_it_should_send_result() {
        let con = wmi_con();
        let stream = AsyncQueryResultStreamInner::new();
        let sink = QuerySink::allocate(stream.clone());
        let p_sink = sink.query_interface::<IWbemObjectSink>().unwrap();
        let mut stream = AsyncQueryResultStream::new(stream, sink);

        let raw_os = con
            .get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#)
            .unwrap();
        let raw_os2 = con
            .get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#)
            .unwrap();
        let ptr: *mut IWbemClassObject = raw_os.inner.as_ptr();
        let ptr2: *mut IWbemClassObject = raw_os2.inner.as_ptr();

        let mut arr = vec![ptr, ptr2];

        // tests on ref count before Indicate call
        unsafe {
            let test_ptr = &ptr;
            let refcount = test_ptr.as_ref().unwrap().AddRef();
            assert_eq!(refcount, 2);
            let refcount = test_ptr.as_ref().unwrap().Release();
            assert_eq!(refcount, 1);
        }

        unsafe {
            p_sink.indicate(arr.len() as i32, arr.as_mut_ptr());
        }
        // tests on ref count after Indicate call
        unsafe {
            let test_ptr = &ptr;
            let refcount = test_ptr.as_ref().unwrap().AddRef();
            assert_eq!(refcount, 3);
            let refcount = test_ptr.as_ref().unwrap().Release();
            assert_eq!(refcount, 2);
        }

        let first = stream.next().await.unwrap().unwrap();

        assert_eq!(first.class().unwrap().as_str(), "Win32_OperatingSystem");

        let second = stream.next().await.unwrap().unwrap();
        assert_eq!(second.class().unwrap().as_str(), "Win32_OperatingSystem");
    }

    #[async_std::test]
    async fn async_it_should_complete_after_set_status_call() {
        let stream = AsyncQueryResultStreamInner::new();
        let sink = QuerySink::allocate(stream.clone());
        let p_sink = sink.query_interface::<IWbemObjectSink>().unwrap();
        let stream = AsyncQueryResultStream::new(stream, sink);

        unsafe {
            p_sink.set_status(
                WBEM_STATUS_COMPLETE as i32,
                0,
                NULL as BSTR,
                NULL as *mut IWbemClassObject,
            );
        }

        let results: Vec<_> = stream.collect().await;

        assert!(results.is_empty());
    }

    #[async_std::test]
    async fn async_it_should_return_e_pointer_after_indicate_call_with_null_pointer() {
        let stream = AsyncQueryResultStreamInner::new();
        let sink = QuerySink::allocate(stream.clone());
        let p_sink = sink.query_interface::<IWbemObjectSink>().unwrap();
        let mut stream = AsyncQueryResultStream::new(stream, sink);

        let mut arr = vec![NULL as *mut IWbemClassObject];
        let result;

        unsafe { result = p_sink.indicate(1, arr.as_mut_ptr()) }
        assert_eq!(result, E_POINTER);

        let item = stream.next().await.unwrap();

        match item {
            Err(WMIError::NullPointerResult) => assert!(true),
            _ => assert!(false),
        }
    }
}
