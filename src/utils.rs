use failure::{Fail};
use winapi::shared::ntdef::HRESULT;

#[derive(Debug, Fail)]
pub enum WMIError {
    #[fail(display = "HRESULT Call failed with: {:#X}", hres)]
    HResultError { hres: HRESULT },
}

pub fn check_hres(hres: HRESULT) -> Result<(), WMIError> {
    if hres < 0 {
        dbg!(hres);
        return Err(WMIError::HResultError { hres });
    }

    Ok(())
}
