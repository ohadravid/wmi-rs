use super::Variant;
use serde::{de, ser};
use std::fmt::{Debug, Display};
use std::string::FromUtf16Error;
use thiserror::Error;
use widestring::NulError;
use winapi::shared::{ntdef::HRESULT, wtypes::VARTYPE};

#[derive(Debug, Error)]
pub enum WMIError {
    #[error("HRESULT Call failed with: {hres:#X}")]
    HResultError { hres: HRESULT },
    #[error("Converting from variant type {0:#X} is not implemented yet")]
    ConvertError(VARTYPE),
    #[error("Invalid bool value: {0:#X}")]
    ConvertBoolError(i16),
    #[error(transparent)]
    ConvertStringError(#[from] FromUtf16Error),
    #[error("Invalid nul value was found: {0:?}")]
    ConvertStringNullError(#[from] NulError<u16>),
    #[error("{0}")]
    SerdeError(String),
    #[error(transparent)]
    DeserializeValueError(#[from] de::value::Error),
    #[error("{0}")]
    Custom(&'static str),
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
