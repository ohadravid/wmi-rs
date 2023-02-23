use crate::{WMIError, WMIResult};
use windows::core::BSTR;

/// A non-null [BSTR]
///
/// [BSTR]: https://docs.microsoft.com/en-us/previous-versions/windows/desktop/automat/bstr
pub(crate) struct BStr(pub BSTR);

impl BStr {
    pub fn from_str(s: &str) -> WMIResult<Self> {
        let value: Vec<u16> = s.encode_utf16().collect();
        match BSTR::from_wide(&value) {
            Ok(v) => Ok(Self(v)),
            Err(_) => Err(WMIError::ConvertAllocateError),
        }
    }
}
