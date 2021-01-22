//! # Async query support
//! This module does not export anything, as it provides additionnal 
//! methods on [`WMIConnection`](WMIConnection)
//!
//! You only have to activate the `async-query` feature flag in Cargo.toml to use them.
//! ```toml
//! wmi = { version = "x.y.z",  features = ["async-query"] }
//! ```
//! 
//! 

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

///
/// ### Aditionnal async methods
/// **Following methods are implemented under the
/// `async-query` feature flag.**
///
impl WMIConnection {
    /// Wrapper for the [ExecQueryAsync](https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execqueryasync)
    /// method. Provides safety checks, and returns results
    /// as a Stream instead of the original Sink.
    ///
    pub fn exec_query_async_native_wrapper(
        &self,
        query: impl AsRef<str>,
    ) -> Result<impl Stream<Item=Result<IWbemClassWrapper, WMIError>>, WMIError> {
        let query_language = BStr::from_str("WQL")?;
        let query = BStr::from_str(query.as_ref())?;

        let (tx, rx) = async_channel::unbounded();
        let p_sink: ComPtr<IWbemObjectSink> = QuerySink::new(tx);

        unsafe {
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
            .exec_query_async_native_wrapper("SELECT OSArchitecture FROM Win32_OperatingSystem")
            .unwrap()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result.len(), 1);
    }
}
