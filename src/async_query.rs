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
    connection::WMIConnection,
    utils::check_hres,
    WMIResult,
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
use serde::de;
use futures::stream::{Stream, TryStreamExt, StreamExt};

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
    ) -> WMIResult<impl Stream<Item=WMIResult<IWbemClassWrapper>>> {
        let query_language = BStr::from_str("WQL")?;
        let query = BStr::from_str(query.as_ref())?;

        let (tx, rx) = async_channel::unbounded();
        // ComPtr keeps an internal RefCount with initial value = 1
        let p_sink: ComPtr<IWbemObjectSink> = QuerySink::new(tx);

        unsafe {
            // As p_sink.RefCount = 1 before this call,
            // p_sink won't be dropped at the end of ExecQueryAsync
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

    /// Async version of [`raw_query`](WMIConnection#method.raw_query)
    /// Execute a free-text query and deserialize the results.
    /// Can be used either with a struct (like `query` and `filtered_query`),
    /// but also with a generic map.
    ///
    /// ```edition2018
    /// # use wmi::*;
    /// # use std::collections::HashMap;
    /// # use futures::executor::block_on;
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// #   block_on(exec_async_query())?;
    /// #   Ok(())
    /// # }

    /// async fn exec_async_query() -> WMIResult<Vec<HashMap<String, Variant>>> {
    ///    use futures::stream::TryStreamExt;
    ///    let con = WMIConnection::new(COMLibrary::new()?.into())?;
    ///    let results: Vec<HashMap<String, Variant>> = 
    ///        con.async_raw_query("SELECT Name FROM Win32_OperatingSystem").await?;
    ///    Ok(results)
    /// }
    /// ```
    pub async fn async_raw_query<T>(&self, query: impl AsRef<str>) -> WMIResult<Vec<T>>
    where
        T: de::DeserializeOwned,
    {
        self
            .exec_query_async_native_wrapper(query)?
            .map(|item| match item {
                Ok(wbem_class_obj) => wbem_class_obj.into_desr(),
                Err(e) => Err(e),
            })
            .try_collect::<Vec<_>>()
            .await
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use crate::tests::fixtures::*;
    use crate::Variant;
    use std::collections::HashMap;
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

    #[async_std::test]
    async fn _async_it_handles_invalid_query() {
        let wmi_con = wmi_con();

        let result = wmi_con
            .exec_query_async_native_wrapper("invalid query")
            .unwrap()
            .collect::<Vec<_>>()
            .await;

            assert_eq!(result.len(), 0);
    }

    #[async_std::test]
    async fn _async_it_provides_raw_query_result() {
        let wmi_con = wmi_con();
        
        let results: Vec<HashMap<String, Variant>> =
            wmi_con.async_raw_query("SELECT * FROM Win32_GroupUser")
            .await.unwrap();

        for res in results {
            match res.get("GroupComponent") {
                Some(Variant::String(s)) => assert!(s != ""),
                _ => assert!(false),
            }

            match res.get("PartComponent") {
                Some(Variant::String(s)) => assert!(s != ""),
                _ => assert!(false),
            }
        }
    }
}
