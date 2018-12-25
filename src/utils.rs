use std::io;
use winapi::shared::ntdef::HRESULT;

pub fn check_hres(hres: HRESULT) -> Result<(), io::Error> {
    if hres < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
