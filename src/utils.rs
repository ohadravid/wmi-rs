use serde::{de, ser};
use std::fmt::{Debug, Display};
use thiserror::Error;
use winapi::shared::{ntdef::HRESULT, wtypes::VARTYPE};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WMIError {
    /// You can find a useful resource for decoding error codes [here](https://docs.microsoft.com/en-us/windows/win32/wmisdk/wmi-error-constants)
    /// (or a github version [here](https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/WmiSdk/wmi-error-constants.md))
    #[error("HRESULT Call failed with: {hres:#X}")]
    HResultError { hres: HRESULT },
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[cfg(feature = "chrono")]
    #[error(transparent)]
    ParseDatetimeError(#[from] chrono::format::ParseError),
    #[cfg(feature = "time")]
    #[error(transparent)]
    ParseOffsetDatetimeError(#[from] time::Error),
    #[error("Converting from variant type {0:#X} is not implemented yet")]
    ConvertError(VARTYPE),
    #[error("{0}")]
    ConvertVariantError(String),
    #[error("Invalid bool value: {0:#X}")]
    ConvertBoolError(i16),
    #[error(transparent)]
    ConvertStringError(#[from] std::string::FromUtf16Error),
    #[error(transparent)]
    ConvertStringUtf16Error(#[from] widestring::error::Utf16Error),
    #[error("Invalid nul value was found: {0:?}")]
    ConvertStringNullError(#[from] widestring::error::NulError<u16>),
    #[error("Expected {0:?} to be at least 21 chars")]
    ConvertDatetimeError(String),
    #[error("Expected {0:?} to be at 25 chars")]
    ConvertDurationError(String),
    #[error("Length {0} was too long to convert")]
    ConvertLengthError(u64),
    #[error("Failed to allocate")]
    ConvertAllocateError,
    #[error("{0}")]
    SerdeError(String),
    #[error(transparent)]
    DeserializeValueError(#[from] de::value::Error),
    #[error("No results returned")]
    ResultEmpty,
    #[error("Null pointer was sent as part of query result")]
    NullPointerResult,
    #[error("Unimplemeted array item in query")]
    UnimplementedArrayItem,
    #[error("Invalid variant {0} during deserialization")]
    InvalidDeserializationVariantError(String),
}

impl de::Error for WMIError {
    #[cold]
    fn custom<T: Display>(msg: T) -> WMIError {
        Self::SerdeError(format!("{}", msg))
    }
}

impl ser::Error for WMIError {
    #[cold]
    fn custom<T: Display>(msg: T) -> WMIError {
        Self::SerdeError(format!("{}", msg))
    }
}

pub fn check_hres(hres: HRESULT) -> WMIResult<()> {
    if hres < 0 {
        return Err(WMIError::HResultError { hres });
    }
    Ok(())
}

/// Alias type for `Result<T, WMIError>`
pub type WMIResult<T> = Result<T, WMIError>;
