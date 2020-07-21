use serde::{de, ser};
use std::fmt::{Debug, Display};
use thiserror::Error;
use winapi::shared::{ntdef::HRESULT, wtypes::VARTYPE};

#[derive(Debug, Error)]
pub enum WMIError {
    #[error("HRESULT Call failed with: {hres:#X}")]
    HResultError { hres: HRESULT },
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseDatetimeError(#[from] chrono::format::ParseError),
    #[error("Converting from variant type {0:#X} is not implemented yet")]
    ConvertError(VARTYPE),
    #[error("{0}")]
    ConvertVariantError(String),
    #[error("Invalid bool value: {0:#X}")]
    ConvertBoolError(i16),
    #[error(transparent)]
    ConvertStringError(#[from] std::string::FromUtf16Error),
    #[error("Invalid nul value was found: {0:?}")]
    ConvertStringNullError(#[from] widestring::NulError<u16>),
    #[error("Expected {0:?} to be at least 21 chars")]
    ConvertDatetimeError(String),
    #[error("Expected {0:?} to be at 25 chars")]
    ConvertDurationError(String),
    #[error("{0}")]
    SerdeError(String),
    #[error(transparent)]
    DeserializeValueError(#[from] de::value::Error),
    #[error("No results returned")]
    ResultEmpty,
    #[error("{0}")]
    Custom(String),
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

pub fn check_hres(hres: HRESULT) -> Result<(), WMIError> {
    if hres < 0 {
        return Err(WMIError::HResultError { hres });
    }

    Ok(())
}
