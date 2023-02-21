use crate::{COMLibrary, WMIConnection, WMIError, WMIResult};

pub mod fixtures {
    use super::*;

    // This way we only setup COM security once per thread during tests.
    thread_local! {
        static COM_LIB: COMLibrary = COMLibrary::without_security().unwrap();
    }

    pub fn wmi_con() -> WMIConnection {
        let com_lib = COM_LIB.with(|com| *com);

        let wmi_con = WMIConnection::new(com_lib).unwrap();

        wmi_con
    }
}

pub fn start_test_program() {
    std::process::Command::new("C:\\Windows\\System32\\cmd.exe")
        .args(["timeout", "1"])
        .spawn()
        .expect("failed to run test program");
}

pub fn ignore_access_denied(result: WMIResult<()>) -> WMIResult<()> {
    use winapi::{shared::ntdef::HRESULT, um::wbemcli::WBEM_E_ACCESS_DENIED};
    if let Err(e) = result {
        if let WMIError::HResultError { hres } = e {
            if hres != WBEM_E_ACCESS_DENIED as HRESULT {
                return Err(e);
            }
        }
    }
    Ok(())
}
