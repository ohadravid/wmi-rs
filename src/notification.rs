use crate::{
    bstr::BStr,
    build_notification_query,
    query_sink::{AsyncQueryResultStream, IWbemObjectSink, QuerySink},
    result_enumerator::{IWbemClassWrapper, QueryResultEnumerator},
    utils::check_hres,
    FilterValue, WMIConnection, WMIResult,
};
use com::{production::ClassAllocation, AbiTransferable};
use futures::{Stream, StreamExt};
use std::{collections::HashMap, ptr, time::Duration};
use winapi::{
    shared::ntdef::NULL,
    um::wbemcli::{IEnumWbemClassObject, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY},
};

///
/// ### Additional notification query methods
///
impl WMIConnection {
    /// Execute the given query to receive events and return an iterator of WMI pointers.
    /// It's better to use the other query methods, since this is relatively low level.
    ///
    pub fn notification_native_wrapper(
        &self,
        query: impl AsRef<str>,
    ) -> WMIResult<QueryResultEnumerator> {
        let query_language = BStr::from_str("WQL")?;
        let query = BStr::from_str(query.as_ref())?;

        let mut p_enumerator = NULL as *mut IEnumWbemClassObject;

        unsafe {
            check_hres((*self.svc()).ExecNotificationQuery(
                query_language.as_bstr(),
                query.as_bstr(),
                (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY) as i32,
                ptr::null_mut(),
                &mut p_enumerator,
            ))?;
        }
        log::trace!("Got enumerator {:?}", p_enumerator);

        Ok(unsafe { QueryResultEnumerator::new(self, p_enumerator) })
    }

    /// Execute a free-text query and deserialize the incoming events.
    /// Returns an iterator of WMIResult\<T\>.
    /// Can be used either with a struct (like `query` and `filtered_query`),
    /// but also with a generic map.
    ///
    /// ```edition2018
    /// # use wmi::*;
    /// # #[cfg(not(feature = "test"))]
    /// # fn main() {}
    /// # #[cfg(feature = "test")]
    /// # fn main() -> wmi::WMIResult<()> {
    /// #   tests::ignore_access_denied(run())
    /// # }
    /// # fn run() -> wmi::WMIResult<()> {
    /// # use std::collections::HashMap;
    /// # let con = WMIConnection::new(COMLibrary::new()?)?;
    /// let iterator = con.raw_notification::<HashMap<String, Variant>>("SELECT ProcessID, ProcessName FROM Win32_ProcessStartTrace")?;
    /// #   Ok(()) // This query will fail when not run as admin
    /// # }
    /// ```
    pub fn raw_notification<'a, T>(
        &'a self,
        query: impl AsRef<str>,
    ) -> WMIResult<impl Iterator<Item = WMIResult<T>> + 'a>
    where
        T: serde::de::DeserializeOwned + 'a,
    {
        let enumerator = self.notification_native_wrapper(query)?;
        let iter = enumerator.map(|item| match item {
            Ok(wbem_class_obj) => wbem_class_obj.into_desr(),
            Err(e) => Err(e),
        });
        Ok(iter)
    }

    /// Subscribe to the T event and return an iterator of WMIResult\<T\>.
    ///
    /// ```edition2018
    /// use wmi::*;
    /// # #[cfg(not(feature = "test"))]
    /// # fn main() {}
    /// # #[cfg(feature = "test")]
    /// # fn main() -> wmi::WMIResult<()> {
    /// #   tests::ignore_access_denied(run())
    /// # }
    /// # fn run() -> wmi::WMIResult<()> {
    /// use serde::Deserialize;
    ///
    /// let con = WMIConnection::new(COMLibrary::new()?)?;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_ProcessStartTrace {
    ///     ProcessID: u32,
    ///     ProcessName: String,
    /// }
    ///
    /// let iterator = con.notification::<Win32_ProcessStartTrace>()?;
    /// #   Ok(()) // This query will fail when not run as admin
    /// # }
    /// ```
    pub fn notification<'a, T>(&'a self) -> WMIResult<impl Iterator<Item = WMIResult<T>> + 'a>
    where
        T: serde::de::DeserializeOwned + 'a,
    {
        let query_text = build_notification_query::<T>(None, None)?;
        self.raw_notification(query_text)
    }

    /// Subscribe to the T event, while filtering according to `filters`.
    /// Returns an iterator of WMIResult\<T\>.
    ///
    /// ```edition2018
    /// # fn main() -> wmi::WMIResult<()> {
    /// # use std::{collections::HashMap, time::Duration};
    /// # use wmi::*;
    /// # let con = WMIConnection::new(COMLibrary::new()?)?;
    /// use serde::Deserialize;
    /// #[derive(Deserialize, Debug)]
    /// struct __InstanceCreationEvent {
    ///     TargetInstance: Win32_Process,
    /// }
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_Process {
    ///     ProcessID: u32,
    /// }
    ///
    /// let mut filters = HashMap::new();
    ///
    /// filters.insert("TargetInstance".to_owned(), FilterValue::is_a::<Win32_Process>()?);
    ///
    /// let iterator = con.filtered_notification::<__InstanceCreationEvent>(&filters, Some(Duration::from_secs(1)))?;

    /// #   Ok(())
    /// # }
    /// ```
    pub fn filtered_notification<'a, T>(
        &'a self,
        filters: &HashMap<String, FilterValue>,
        within: Option<Duration>,
    ) -> WMIResult<impl Iterator<Item = WMIResult<T>> + 'a>
    where
        T: serde::de::DeserializeOwned + 'a,
    {
        let query_text = build_notification_query::<T>(Some(filters), within)?;
        self.raw_notification(query_text)
    }

    /// Wrapper for the [ExecNotificationQueryAsync](https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execnotificationqueryasync)
    /// method. Provides safety checks, and returns results
    /// as a stream instead of the original Sink.
    ///
    pub fn async_notification_native_wrapper(
        &self,
        query: impl AsRef<str>,
    ) -> WMIResult<impl Stream<Item = WMIResult<IWbemClassWrapper>>> {
        let query_language = BStr::from_str("WQL")?;
        let query = BStr::from_str(query.as_ref())?;

        let stream = AsyncQueryResultStream::new();
        // The internal RefCount has initial value = 1.
        let p_sink: ClassAllocation<QuerySink> = QuerySink::allocate(stream.clone());
        let p_sink_handle = IWbemObjectSink::from(&**p_sink);

        unsafe {
            // As p_sink's RefCount = 1 before this call,
            // p_sink won't be dropped at the end of ExecNotificationQueryAsync
            check_hres((*self.svc()).ExecNotificationQueryAsync(
                query_language.as_bstr(),
                query.as_bstr(),
                0,
                ptr::null_mut(),
                p_sink_handle.get_abi().as_ptr() as *mut _,
            ))?;
        }

        Ok(stream)
    }

    /// Async version of [`raw_notification`](WMIConnection#method.raw_notification)
    /// Execute a free-text query and deserialize the incoming events.
    /// Returns a stream of WMIResult\<T\>.
    /// Can be used either with a struct (like `query` and `filtered_query`),
    /// but also with a generic map.
    ///
    /// ```edition2018
    /// # use wmi::*;
    /// # use std::collections::HashMap;
    /// # use futures::{executor::block_on, StreamExt};
    /// # #[cfg(not(feature = "test"))]
    /// # fn main() {}
    /// # #[cfg(feature = "test")]
    /// # fn main() -> wmi::WMIResult<()> {
    /// #   tests::ignore_access_denied(block_on(exec_async_query()))
    /// # }
    /// #
    /// # async fn exec_async_query() -> WMIResult<()> {
    /// # let con = WMIConnection::new(COMLibrary::new()?)?;
    /// let mut stream = con.async_raw_notification::<HashMap<String, Variant>>("SELECT ProcessID, ProcessName FROM Win32_ProcessStartTrace")?;
    /// # let event = stream.next().await.unwrap()?;
    /// #   Ok(()) // This query will fail when not run as admin
    /// # }
    /// ```
    pub fn async_raw_notification<T>(
        &self,
        query: impl AsRef<str>,
    ) -> WMIResult<impl Stream<Item = WMIResult<T>>>
    where
        T: serde::de::DeserializeOwned,
    {
        let stream = self
            .async_notification_native_wrapper(query)?
            .map(|item| match item {
                Ok(wbem_class_obj) => wbem_class_obj.into_desr(),
                Err(e) => Err(e),
            });
        Ok(stream)
    }

    /// Subscribe to the T event and return a stream of WMIResult\<T\>.
    ///
    /// ```edition2018
    /// # use wmi::*;
    /// # use std::collections::HashMap;
    /// # use futures::executor::block_on;
    /// # #[cfg(not(feature = "test"))]
    /// # fn main() {}
    /// # #[cfg(feature = "test")]
    /// # fn main() -> wmi::WMIResult<()> {
    /// #   tests::ignore_access_denied(block_on(exec_async_query()))
    /// # }
    /// #
    /// # async fn exec_async_query() -> WMIResult<()> {
    /// # let con = WMIConnection::new(COMLibrary::new()?)?;
    /// use futures::StreamExt;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_ProcessStartTrace {
    ///     ProcessID: u32,
    ///     ProcessName: String,
    /// }
    ///
    /// let mut stream = con.async_notification::<Win32_ProcessStartTrace>()?;
    ///
    /// let event = stream.next().await.unwrap()?;
    /// #   Ok(()) // This query will fail when not run as admin
    /// # }
    /// ```
    pub fn async_notification<T>(&self) -> WMIResult<impl Stream<Item = WMIResult<T>>>
    where
        T: serde::de::DeserializeOwned,
    {
        let query_text = build_notification_query::<T>(None, None)?;
        self.async_raw_notification(query_text)
    }

    /// Subscribe to the T event, while filtering according to `filters`.
    /// Returns a stream of WMIResult\<T\>.
    ///
    /// ```edition2018
    /// # use wmi::*;
    /// # use futures::{future::FutureExt, select};
    /// # fn main() -> wmi::WMIResult<()> {
    /// #   async_std::task::block_on(async {
    /// #       select! { // End in 3 seconds or on event.
    /// #           () = async_std::task::sleep(std::time::Duration::from_secs(3)).fuse() => Ok(()),
    /// #           r = exec_async_query().fuse() => r
    /// #       }
    /// #   })
    /// # }
    /// #
    /// # async fn exec_async_query() -> WMIResult<()> {
    /// # use std::{collections::HashMap, time::Duration};
    /// # let con = WMIConnection::new(COMLibrary::new()?)?;
    /// use futures::StreamExt;
    /// use serde::Deserialize;
    /// #[derive(Deserialize, Debug)]
    /// struct __InstanceCreationEvent {
    ///     TargetInstance: Win32_Process,
    /// }
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_Process {
    ///     ProcessID: u32,
    /// }
    ///
    /// let mut filters = HashMap::new();
    ///
    /// filters.insert("TargetInstance".to_owned(), FilterValue::is_a::<Win32_Process>()?);
    ///
    /// let mut stream = con.async_filtered_notification::<__InstanceCreationEvent>(&filters, Some(Duration::from_secs(1)))?;
    ///
    /// let event = stream.next().await.unwrap()?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn async_filtered_notification<T>(
        &self,
        filters: &HashMap<String, FilterValue>,
        within: Option<Duration>,
    ) -> WMIResult<impl Stream<Item = WMIResult<T>>>
    where
        T: serde::de::DeserializeOwned,
    {
        let query_text = build_notification_query::<T>(Some(filters), within)?;
        self.async_raw_notification(query_text)
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::fixtures::*, FilterValue, WMIError};
    use futures::StreamExt;
    use serde::Deserialize;
    use std::{collections::HashMap, time::Duration};
    use winapi::{shared::ntdef::HRESULT, um::wbemcli::WBEM_E_UNPARSABLE_QUERY};

    #[cfg(feature = "chrono")]
    use chrono::Datelike;

    const TEST_QUERY: &str =
        "SELECT * FROM __InstanceModificationEvent WHERE TargetInstance ISA 'Win32_LocalTime'";

    pub fn notification_filters() -> HashMap<String, FilterValue> {
        let mut map = HashMap::<String, FilterValue>::new();
        map.insert(
            "TargetInstance".to_owned(),
            FilterValue::is_a::<LocalTime>().unwrap(),
        );
        map
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "__InstanceModificationEvent")]
    #[serde(rename_all = "PascalCase")]
    pub struct InstanceModification {
        target_instance: LocalTime,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_LocalTime")]
    #[serde(rename_all = "PascalCase")]
    pub struct LocalTime {
        year: u32,
    }

    #[test]
    fn it_works() {
        let wmi_con = wmi_con();

        let mut enumerator = wmi_con.notification_native_wrapper(TEST_QUERY).unwrap();

        let res = enumerator.next().unwrap();
        let w = res.unwrap();
        let mut props = w.list_properties().unwrap();

        props.sort();

        assert_eq!(props.len(), 4);
        assert_eq!(props[..2], ["PreviousInstance", "SECURITY_DESCRIPTOR"]);
        assert_eq!(props[props.len() - 2..], ["TIME_CREATED", "TargetInstance"]);
    }

    #[test]
    fn it_fails_gracefully() {
        let wmi_con = wmi_con();

        let mut enumerator = wmi_con.notification_native_wrapper("SELECT NoSuchField FROM __InstanceModificationEvent WHERE TargetInstance ISA 'Win32_LocalTime'").unwrap();

        let res = enumerator.next().unwrap();
        assert!(res.is_ok());

        let props = res.unwrap().list_properties().unwrap();
        assert_eq!(props.len(), 0);
    }

    #[test]
    fn it_fails_gracefully_with_invalid_sql() {
        let wmi_con = wmi_con();

        let result = wmi_con.notification_native_wrapper("42");

        match result {
            Ok(_) => assert!(false),
            Err(wmi_err) => match wmi_err {
                WMIError::HResultError { hres } => {
                    assert_eq!(hres, WBEM_E_UNPARSABLE_QUERY as HRESULT)
                }
                _ => assert!(false),
            },
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn it_can_run_raw_notification() {
        let wmi_con = wmi_con();

        let mut iterator = wmi_con
            .raw_notification::<InstanceModification>(TEST_QUERY)
            .unwrap();

        let local_time = iterator.next().unwrap();
        assert!(local_time.is_ok());

        let local_time = local_time.unwrap().target_instance;
        assert_eq!(local_time.year as i32, chrono::Local::now().year());
    }

    #[test]
    #[cfg(feature = "time")]
    fn it_can_run_raw_notification_on_time_crate() {
        let wmi_con = wmi_con();

        let mut iterator = wmi_con
            .raw_notification::<InstanceModification>(TEST_QUERY)
            .unwrap();

        let local_time = iterator.next().unwrap();
        assert!(local_time.is_ok());

        let local_time = local_time.unwrap().target_instance;
        assert_eq!(
            local_time.year as i32,
            time::OffsetDateTime::now_utc().year()
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn it_can_run_filtered_notification() {
        let wmi_con = wmi_con();

        let mut iterator = wmi_con
            .filtered_notification::<InstanceModification>(
                &notification_filters(),
                Some(Duration::from_secs_f32(0.1)),
            )
            .unwrap();

        let local_time = iterator.next().unwrap();
        assert!(local_time.is_ok());

        let local_time = local_time.unwrap().target_instance;
        assert_eq!(local_time.year as i32, chrono::Local::now().year());
    }

    #[test]
    #[cfg(feature = "time")]
    fn it_can_run_filtered_notification_on_time_crate() {
        let wmi_con = wmi_con();

        let mut iterator = wmi_con
            .filtered_notification::<InstanceModification>(
                &notification_filters(),
                Some(Duration::from_secs_f32(0.1)),
            )
            .unwrap();

        let local_time = iterator.next().unwrap();
        assert!(local_time.is_ok());

        let local_time = local_time.unwrap().target_instance;
        assert_eq!(
            local_time.year as i32,
            time::OffsetDateTime::now_utc().year()
        );
    }

    #[async_std::test]
    async fn async_it_works_async_std() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .async_notification_native_wrapper(TEST_QUERY)
            .unwrap()
            .next()
            .await
            .unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn async_it_works_async_tokio() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .async_notification_native_wrapper(TEST_QUERY)
            .unwrap()
            .next()
            .await
            .unwrap();

        assert!(result.is_ok());
    }

    #[async_std::test]
    async fn async_it_handles_invalid_query() {
        let wmi_con = wmi_con();

        let result = wmi_con.async_notification_native_wrapper("Invalid Query");

        assert!(result.is_err());
        if let WMIError::HResultError { hres } = result.err().unwrap() {
            assert_eq!(hres, WBEM_E_UNPARSABLE_QUERY as HRESULT)
        } else {
            assert!(false, "Invalid WMIError type");
        }
    }

    #[async_std::test]
    #[cfg(feature = "chrono")]
    async fn async_it_provides_raw_notification_result() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .async_raw_notification::<InstanceModification>(TEST_QUERY)
            .unwrap()
            .next()
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().target_instance.year as i32,
            chrono::Local::now().year()
        )
    }

    #[async_std::test]
    #[cfg(feature = "time")]
    async fn async_it_provides_raw_notification_result_on_time_crate() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .async_raw_notification::<InstanceModification>(TEST_QUERY)
            .unwrap()
            .next()
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().target_instance.year as i32,
            time::OffsetDateTime::now_utc().year()
        )
    }

    #[async_std::test]
    #[cfg(feature = "chrono")]
    async fn async_it_provides_filtered_notification_result() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .async_filtered_notification::<InstanceModification>(
                &notification_filters(),
                Some(Duration::from_secs_f32(0.1)),
            )
            .unwrap()
            .next()
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().target_instance.year as i32,
            chrono::Local::now().year()
        )
    }

    #[async_std::test]
    #[cfg(feature = "time")]
    async fn async_it_provides_filtered_notification_result_on_time_crate() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .async_filtered_notification::<InstanceModification>(
                &notification_filters(),
                Some(Duration::from_secs_f32(0.1)),
            )
            .unwrap()
            .next()
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().target_instance.year as i32,
            time::OffsetDateTime::now_utc().year()
        )
    }
}
