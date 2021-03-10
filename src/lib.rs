//! # WMI-rs
//!
//! [WMI] is a management API for Windows-based operating systems.
//! This crate provides a high level Rust API focused around data retrieval (vs. making changes to
//! the system and watching for event which are also supported by WMI).
//!
//! This crate also uses `serde` to transform pointers to WMI class objects into plain Rust structs.
//!
//! All data is copied to Owning data structures, so the final structs are not tied in any way to
//! the original WMI object (refer to MSDN's [Creating a WMI Application Using C++] to learn more about how data is handled by WMI).
//!
//! Before using WMI, a connection must be created.
//!
//! ```rust
//! # fn main() -> Result<(), wmi::WMIError> {
//! use wmi::{COMLibrary, WMIConnection};
//! let com_con = COMLibrary::new()?;
//! let wmi_con = WMIConnection::new(com_con.into())?;
//! #   Ok(())
//! # }
//! ```
//!
//! There are multiple ways to get data from the OS using this crate.
//!
//! # Operating on untyped Variants
//!
//! WMI data model is based on COM's [`VARIANT`] Type, which is a struct capable of holding
//! many types of data.
//!
//! This crate provides the analogous [`Variant`][Variant] enum.
//!
//! Using this enum, we can execute a simple WMI query and inspect the results.
//!
//! ```edition2018
//! # fn main() -> Result<(), wmi::WMIError> {
//! use wmi::*;
//! let wmi_con = WMIConnection::new(COMLibrary::new()?.into())?;
//! use std::collections::HashMap;
//! use wmi::Variant;
//! let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem").unwrap();
//!
//! for os in results {
//!     println!("{:#?}", os);
//! }
//! #   Ok(())
//! # }
//! ```
//!
//! # Using strongly typed data structures
//!
//! Using `serde`, it is possible to return a struct representing the the data.
//!
//! ```edition2018
//! # fn main() -> Result<(), wmi::WMIError> {
//! # use wmi::*;
//! # let wmi_con = WMIConnection::new(COMLibrary::new()?.into())?;
//! use serde::Deserialize;
//! use wmi::WMIDateTime;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(rename = "Win32_OperatingSystem")]
//! #[serde(rename_all = "PascalCase")]
//! struct OperatingSystem {
//!     caption: String,
//!     debug: bool,
//!     last_boot_up_time: WMIDateTime,
//! }
//!
//! let results: Vec<OperatingSystem> = wmi_con.query()?;
//!
//! for os in results {
//!     println!("{:#?}", os);
//! }
//! #   Ok(())
//! # }
//! ```
//!
//! Because the name of the struct given to `serde` matches the [WMI class] name, the SQL query
//! can be inferred.
//!
//! [WMI]: https://docs.microsoft.com/en-us/windows/desktop/wmisdk/about-wmi
//! [Creating a WMI Application Using C++]: https://docs.microsoft.com/en-us/windows/desktop/wmisdk/creating-a-wmi-application-using-c-
//! [`VARIANT`]: https://docs.microsoft.com/en-us/windows/desktop/api/oaidl/ns-oaidl-tagvariant
//! [WMI class]: https://docs.microsoft.com/en-us/windows/desktop/cimwin32prov/win32-operatingsystem
//!
//! # Internals
//!
//! [`WMIConnection`](WMIConnection) is used to create and execute a WMI query, returning
//! [`IWbemClassWrapper`](result_enumerator::IWbemClassWrapper) which is a wrapper for a WMI object pointer.
//!
//! Then, [`from_wbem_class_obj`](de::wbem_class_de::from_wbem_class_obj) is used to create a Rust struct with the equivalent data.
//!
//! Deserializing data from WMI and into Rust is done via `serde` and is implemented in the [`de`][de] module.
//! More info can be found in `serde`'s documentation about [writing a data format].
//! The deserializer will either use the field names defined on the output struct,
//! or retrieve all field names from WMI if the output is a `HashMap`.
//!
//! [writing a data format]: https://serde.rs/data-format.html
//!
//! There are two main data structures (other than pointers to object) which convert native data to Rust data structures:
//! [`Variant`](Variant) and [`SafeArrayAccessor`](safearray::SafeArrayAccessor).
//!
//! Most native objects has an equivalent wrapper struct which implements `Drop` for that data.
//!
//! # Async Query
//!
//! Async queries are available behind the feature flag `async-query`.
//! In Cargo.toml:
//! ```toml
//! wmi = { version = "x.y.z",  features = ["async-query"] }
//! ```
//! You now have access to additional methods on [`WMIConnection`](WMIConnection#additional-async-methods).
//!
//! ```edition2018
//! # use futures::executor::block_on;
//! # block_on(exec_async_query()).unwrap();
//! # async fn exec_async_query() -> Result<(), wmi::WMIError> {
//! use wmi::*;
//! use futures::StreamExt;
//! let wmi_con = WMIConnection::new(COMLibrary::new()?.into())?;
//! let results = wmi_con
//!     .exec_query_async_native_wrapper("SELECT OSArchitecture FROM Win32_OperatingSystem")?
//!     .collect::<Vec<_>>().await;
//! #   Ok(())
//! # }
//! ```
//! It it also possible to return a struct representing the the data.
//!
//! ```edition2018
//! # use futures::executor::block_on;
//! # block_on(exec_async_query()).unwrap();
//! # async fn exec_async_query() -> Result<(), wmi::WMIError> {
//! use wmi::*;
//! let wmi_con = WMIConnection::new(COMLibrary::new()?.into())?;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(rename = "Win32_OperatingSystem")]
//! #[serde(rename_all = "PascalCase")]
//! struct OperatingSystem {
//!     caption: String,
//!     debug: bool,
//!     last_boot_up_time: WMIDateTime,
//! }
//!
//! let results: Vec<OperatingSystem> = wmi_con.async_query().await?;
//!
//! for os in results {
//!     println!("{:#?}", os);
//! }
//! #   Ok(())
//! # }
//! ```
//!
//!
//!
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_unsafe)]
#![cfg(windows)]

mod bstr;
pub mod connection;
pub mod datetime;
pub mod de;
pub mod duration;
pub mod query;
pub mod result_enumerator;
pub mod safearray;
pub mod utils;
pub mod variant;

#[cfg(feature = "async-query")]
pub mod async_query;
// Keep QuerySink implementation private
#[cfg(feature = "async-query")]
pub(crate) mod query_sink;

#[cfg(any(test, feature = "test"))]
pub mod tests;

use bstr::BStr;
pub use query::{FilterValue, build_query};
pub use connection::{COMLibrary, WMIConnection};
pub use datetime::WMIDateTime;
pub use duration::WMIDuration;
pub use utils::{WMIError, WMIResult};
pub use variant::Variant;

// Cannot use `cfg(test)` here since `rustdoc` won't look at it.
#[cfg(debug_assertions)]
mod test_readme {
    macro_rules! calculated_doc {
        ($doc:expr, $id:ident) => {
            #[doc = $doc]
            enum $id {}
        }
    }

    calculated_doc!(include_str!("../README.md"), _DoctestReadme);
}