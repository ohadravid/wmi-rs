use crate::BStr;
use crate::result_enumerator::IWbemClassWrapper;
use crate::{
    connection::WMIConnection, utils::check_hres, WMIError,
};
use crate::query_sink::QuerySink;
use std::ptr;
use winapi::{
    um::{
        wbemcli::IWbemObjectSink,
        wbemcli::{
            WBEM_FLAG_BIDIRECTIONAL,
        },
    },
};
use wio::com::ComPtr;
use futures::stream::Stream;

impl WMIConnection {
    /// Execute the given query in async way, returns Stream of result.
    ///
    pub fn exec_async_query_native_wrapper(
        &self,
        query: impl AsRef<str>,
    ) -> Result<impl Stream<Item=Result<IWbemClassWrapper, WMIError>>, WMIError> {
        let query_language = BStr::from_str("WQL")?;
        let query = BStr::from_str(query.as_ref())?;

        let (tx, rx) = async_channel::unbounded();
        let p_sink: ComPtr<IWbemObjectSink> = QuerySink::new(tx);

        unsafe {
            // FIXME hack the RefCount
            p_sink.AddRef();
            p_sink.AddRef();
            p_sink.AddRef();
            p_sink.AddRef();

            check_hres((*self.svc()).ExecQueryAsync(
                query_language.as_bstr(),
                query.as_bstr(),
                WBEM_FLAG_BIDIRECTIONAL as i32,
                ptr::null_mut(),
                p_sink.as_raw(),
            ))?;
            
        }

        Ok(rx)
    }
    
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use crate::tests::fixtures::*;
    use futures::stream::StreamExt;

    #[async_std::test]
    async fn _async_it_works_async() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .exec_async_query_native_wrapper("SELECT OSArchitecture FROM Win32_OperatingSystem")
            .unwrap()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result.len(), 1);
    }
}
