use thiserror::Error;
use winapi::shared::ntdef::HRESULT;

#[derive(Debug, Error)]
pub enum WMIError {
    #[error("HRESULT Call failed with: {hres:#X}")]
    HResultError { hres: HRESULT },
}

pub fn check_hres(hres: HRESULT) -> Result<(), WMIError> {
    if hres < 0 {
        return Err(WMIError::HResultError { hres });
    }

    Ok(())
}
