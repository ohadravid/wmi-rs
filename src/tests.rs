use crate::{COMLibrary, WMIConnection};

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
