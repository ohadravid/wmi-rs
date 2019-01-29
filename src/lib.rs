#![feature(ptr_internals, custom_attribute)]

pub mod connection;
pub mod de;
pub mod query;
pub mod utils;
pub mod error;
pub mod variant;
pub mod datetime;
pub mod safearray;
pub mod consts;

#[cfg(test)]
pub mod tests;

pub use de::wbem_class_de::from_wbem_class_obj;
pub use connection::{COMLibrary, WMIConnection};
pub use variant::Variant;
pub use datetime::WMIDateTime;