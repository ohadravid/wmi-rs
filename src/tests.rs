use crate::COMLibrary;
use crate::WMIConnection;

pub mod fixtures {
    use super::*;
    pub fn wmi_con() -> WMIConnection {
        WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap()
    }
}
