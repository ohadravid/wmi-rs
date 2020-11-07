use crate::WMIError;
use winapi::{
    shared::ntdef::LPCWSTR,
    shared::wtypes::BSTR,
    shared::wtypesbase::OLECHAR,
    um::oleauto::*,
};
use std::convert::TryFrom;
use std::ptr::{null, NonNull};
use std::ops::Drop;

/// A non-null [BSTR]
///
/// [BSTR]:         https://docs.microsoft.com/en-us/previous-versions/windows/desktop/automat/bstr
pub(crate) struct BStr(NonNull<OLECHAR>);

impl Drop for BStr {
    fn drop(&mut self) {
        unsafe { SysFreeString(self.as_bstr()) }
    }
}

impl BStr {
    pub fn from_str(s: &str) -> Result<Self, WMIError> {
        let len = s.encode_utf16().count();
        let len32 = u32::try_from(len).map_err(|_| WMIError::ConvertLengthError(len as _))?;

        let bstr = BStr(NonNull::new(unsafe { SysAllocStringLen(null(), len32) }).ok_or(WMIError::ConvertAllocateError)?);
        for (i, cu) in s.encode_utf16().enumerate() {
            unsafe { std::ptr::write(bstr.0.as_ptr().add(i), cu) };
        }
        Ok(bstr)
    }

    pub fn as_bstr(&self) -> BSTR {
        self.0.as_ptr()
    }

    pub fn as_lpcwstr(&self) -> LPCWSTR {
        self.0.as_ptr()
    }
}
