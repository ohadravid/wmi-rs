use crate::COMLibrary;
use crate::WMIConnection;

pub mod fixtures {
    use super::*;
    use lazy_static::lazy_static;
    use std::rc::Rc;

    // This way we only setup COM security once during tests.
    lazy_static! {
        static ref COM_LIB: COMLibrary = { COMLibrary::new().unwrap() };
    }

    pub fn wmi_con() -> WMIConnection {
        let wmi_con = WMIConnection::new(COMLibrary::without_security().unwrap().into()).unwrap();

        wmi_con
    }
}
