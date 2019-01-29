use crate::utils::check_hres;
use failure::{Error};
use log::{debug};
use std::ptr;
use std::ptr::Unique;
use std::rc::Rc;
use widestring::{WideCString};
use winapi::{
    shared::{
        rpcdce::{
            RPC_C_AUTHN_LEVEL_CALL,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_AUTHN_WINNT,
            RPC_C_AUTHZ_NONE,
            RPC_C_IMP_LEVEL_IMPERSONATE
        },
        ntdef::NULL,
        wtypesbase::CLSCTX_INPROC_SERVER
    },
    um::{
        combaseapi::{
            CoCreateInstance,
            CoInitializeEx,
            CoInitializeSecurity,
            CoSetProxyBlanket,
            CoUninitialize
        },
        objbase::COINIT_MULTITHREADED,
        objidl::EOAC_NONE,
        wbemcli::{
            CLSID_WbemLocator, IID_IWbemLocator, IWbemLocator,
            IWbemServices,
        }
    }
};

pub struct COMLibrary {}

/// Initialize COM.
///
/// COM will be `CoUninitialize`d after this object is dropped.
///
impl COMLibrary {
    /// `CoInitialize`s the COM library for use by the calling thread.
    ///
    pub fn new() -> Result<Self, Error> {
        unsafe { check_hres(CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED))? }

        let instance = Self {};

        instance.init_security()?;

        Ok(instance)
    }

    /// `CoInitialize`s the COM library for use by the calling thread, but without setting the security context.
    ///
    pub fn without_security() -> Result<Self, Error> {
        unsafe { check_hres(CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED))? }

        let instance = Self {};

        Ok(instance)
    }

    fn init_security(&self) -> Result<(), Error> {
        unsafe {
            check_hres(CoInitializeSecurity(
                NULL,
                -1, // let COM choose.
                ptr::null_mut(),
                NULL,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                NULL,
                EOAC_NONE,
                NULL,
            ))?;
        };

        Ok(())
    }
}

impl Drop for COMLibrary {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub struct WMIConnection {
    com_con: Rc<COMLibrary>,
    p_loc: Option<Unique<IWbemLocator>>,
    p_svc: Option<Unique<IWbemServices>>,
}

/// A connection to the local WMI provider, which provides querying capabilities.
///
/// Currently does not support remote providers (e.g connecting to other computers).
///
impl WMIConnection {
    pub fn new(com_lib: Rc<COMLibrary>) -> Result<Self, Error> {
        let mut instance = Self {
            com_con: com_lib,
            p_loc: None,
            p_svc: None,
        };

        instance.create_locator()?;

        instance.create_services()?;

        instance.set_proxy()?;

        Ok(instance)
    }

    pub fn svc(&self) -> *mut IWbemServices {
        self.p_svc.unwrap().as_ptr()
    }

    fn loc(&self) -> *mut IWbemLocator {
        self.p_loc.unwrap().as_ptr()
    }

    fn create_locator(&mut self) -> Result<(), Error> {
        debug!("Calling CoCreateInstance for CLSID_WbemLocator");

        let mut p_loc = NULL;

        unsafe {
            check_hres(CoCreateInstance(
                &CLSID_WbemLocator,
                ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IWbemLocator,
                &mut p_loc,
            ))?;
        }

        self.p_loc = Unique::new(p_loc as *mut IWbemLocator);

        debug!("Got locator {:?}", self.p_loc);

        Ok(())
    }

    fn create_services(&mut self) -> Result<(), Error> {
        debug!("Calling ConnectServer");

        let mut p_svc = ptr::null_mut::<IWbemServices>();

        let object_path = "ROOT\\CIMV2";
        let mut object_path_bstr = WideCString::from_str(object_path)?;

        unsafe {
            check_hres((*self.loc()).ConnectServer(
                object_path_bstr.as_ptr() as *mut _,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut p_svc,
            ))?;
        }

        self.p_svc = Unique::new(p_svc as *mut IWbemServices);

        debug!("Got service {:?}", self.p_svc);

        Ok(())
    }

    fn set_proxy(&self) -> Result<(), Error> {
        debug!("Calling CoSetProxyBlanket");

        unsafe {
            check_hres(CoSetProxyBlanket(
                self.svc() as _,             // Indicates the proxy to set
                RPC_C_AUTHN_WINNT,           // RPC_C_AUTHN_xxx
                RPC_C_AUTHZ_NONE,            // RPC_C_AUTHZ_xxx
                ptr::null_mut(),             // Server principal name
                RPC_C_AUTHN_LEVEL_CALL,      // RPC_C_AUTHN_LEVEL_xxx
                RPC_C_IMP_LEVEL_IMPERSONATE, // RPC_C_IMP_LEVEL_xxx
                NULL,                        // client identity
                EOAC_NONE,                   // proxy capabilities
            ))?;
        }

        Ok(())
    }
}

impl Drop for WMIConnection {
    fn drop(&mut self) {
        if let Some(svc) = self.p_svc {
            unsafe {
                (*svc.as_ptr()).Release();
            }
        }

        if let Some(loc) = self.p_loc {
            unsafe {
                (*loc.as_ptr()).Release();
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        let p_svc = wmi_con.svc();

        assert_eq!(p_svc.is_null(), false);
    }
}
