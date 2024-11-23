use std::collections::HashMap;

use serde::Serialize;
use windows_core::{BSTR, VARIANT};

use crate::{WMIConnection, WMIResult};

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(untagged)]
pub enum ContextValueType {
    String(String),
    I4(i32),
    R8(f64),
    Bool(bool),
}

impl From<ContextValueType> for VARIANT {
    fn from(value: ContextValueType) -> Self {
        match value {
            ContextValueType::Bool(b) => Self::from(b),
            ContextValueType::I4(i4) => Self::from(i4),
            ContextValueType::R8(r8) => Self::from(r8),
            ContextValueType::String(str) => Self::from(BSTR::from(str)),
        }
    }
}

impl WMIConnection {
    /// Sets the specified named context values for use in providing additional context information to queries.
    ///
    /// Note the context values will persist across subsequent queries until [`WMIConnection::clear_ctx_values`] is called.
    pub fn set_ctx_values(
        &mut self,
        ctx_values: HashMap<String, ContextValueType>,
    ) -> WMIResult<()> {
        for (k, v) in ctx_values {
            let key = BSTR::from(k);
            let value = v.clone().into();
            unsafe { self.ctx.SetValue(&key, 0, &value)? };
        }

        Ok(())
    }

    /// Clears all named values from the underlying context object.
    pub fn clear_ctx_values(&mut self) -> WMIResult<()> {
        unsafe { self.ctx.DeleteAll().map_err(Into::into) }
    }
}

macro_rules! impl_from_type {
    ($target_type:ty, $variant:ident) => {
        impl From<$target_type> for ContextValueType {
            fn from(value: $target_type) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}

impl_from_type!(&str, String);
impl_from_type!(i32, I4);
impl_from_type!(f64, R8);
impl_from_type!(bool, Bool);

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::COMLibrary;
    use serde::Deserialize;

    #[test]
    fn verify_ctx_values_used() {
        let com_con = COMLibrary::new().unwrap();
        let mut wmi_con =
            WMIConnection::with_namespace_path("ROOT\\StandardCimv2", com_con).unwrap();

        #[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug)]
        struct MSFT_NetAdapter {
            InterfaceName: String,
        }

        let mut orig_adapters = wmi_con.query::<MSFT_NetAdapter>().unwrap();
        assert!(!orig_adapters.is_empty());

        let mut ctx_values = HashMap::new();
        ctx_values.insert("IncludeHidden".into(), true.into());
        wmi_con.set_ctx_values(ctx_values).unwrap();

        // With 'IncludeHidden' set to 'true', expect the response to contain additional adapters
        let all_adapters = wmi_con.query::<MSFT_NetAdapter>().unwrap();
        assert!(all_adapters.len() > orig_adapters.len());

        wmi_con.clear_ctx_values().unwrap();
        let mut adapters = wmi_con.query::<MSFT_NetAdapter>().unwrap();
        adapters.sort();
        orig_adapters.sort();
        assert_eq!(adapters, orig_adapters);
    }
}
