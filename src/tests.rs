use crate::{COMLibrary, WMIConnection, WMIResult, WMIError};

pub mod fixtures {
    use super::*;

    // This way we only setup COM security once per thread during tests.
    thread_local! {
        static COM_LIB: COMLibrary = COMLibrary::without_security().unwrap();
    }

    pub fn wmi_con() -> WMIConnection {
        let com_lib = COM_LIB.with(|com| *com);

        WMIConnection::new(com_lib).unwrap()
    }
}

pub fn start_test_program() {
    std::process::Command::new("C:\\Windows\\System32\\cmd.exe")
        .args(["timeout", "1"])
        .spawn()
        .expect("failed to run test program");
}

pub fn ignore_access_denied(result: WMIResult<()>) -> WMIResult<()> {
    use windows::Win32::System::Wmi::WBEM_E_ACCESS_DENIED;

    if let Err(e) = result {
        if let WMIError::HResultError { hres } = e {
            if hres != WBEM_E_ACCESS_DENIED.0 {
                return Err(e);
            }
        }
    }
    Ok(())
}
