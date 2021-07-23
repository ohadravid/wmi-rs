use crate::utils::{check_hres, WMIError};
use crate::BStr;
use log::debug;
use std::ptr;
use std::ptr::NonNull;
use std::rc::Rc;
use winapi::{
    shared::{
        ntdef::NULL,
        rpcdce::{
            RPC_C_AUTHN_LEVEL_CALL, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE,
            RPC_C_IMP_LEVEL_IMPERSONATE,
        },
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{
            CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket,
            CoUninitialize,
        },
        objbase::COINIT_MULTITHREADED,
        objidl::EOAC_NONE,
        wbemcli::{CLSID_WbemLocator, IID_IWbemLocator, IWbemLocator, IWbemServices},
    },
};

pub struct COMLibrary {}

/// Initialize COM.
///
/// COM will be `CoUninitialize`d after this object is dropped.
///
impl COMLibrary {
    /// `CoInitialize`s the COM library for use by the calling thread.
    ///
    pub fn new() -> Result<Self, WMIError> {
        unsafe { check_hres(CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED))? }

        let instance = Self {};

        instance.init_security()?;

        Ok(instance)
    }

    /// `CoInitialize`s the COM library for use by the calling thread, but without setting the security context.
    ///
    pub fn without_security() -> Result<Self, WMIError> {
        unsafe { check_hres(CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED))? }

        let instance = Self {};

        Ok(instance)
    }

    fn init_security(&self) -> Result<(), WMIError> {
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
    _com_con: Option<Rc<COMLibrary>>,
    p_loc: Option<NonNull<IWbemLocator>>,
    p_svc: Option<NonNull<IWbemServices>>,
}

/// A connection to the local WMI provider, which provides querying capabilities.
///
/// Currently does not support remote providers (e.g connecting to other computers).
///
impl WMIConnection {
    fn create_and_set_proxy(&mut self, namespace_path: Option<&str>) -> Result<(), WMIError> {
        self.create_locator()?;

        self.create_services(namespace_path.unwrap_or("ROOT\\CIMV2"))?;

        self.set_proxy()?;

        Ok(())
    }

    /// Creates a connection with a default `CIMV2` namespace path.
    pub fn new(com_lib: Rc<COMLibrary>) -> Result<Self, WMIError> {
        Self::with_namespace_path("ROOT\\CIMV2", com_lib)
    }

    /// Creates a connection with the given namespace path.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// let wmi_con = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", COMLibrary::new()?.into())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_namespace_path(
        namespace_path: &str,
        com_lib: Rc<COMLibrary>,
    ) -> Result<Self, WMIError> {
        let mut instance = Self {
            _com_con: Some(com_lib),
            p_loc: None,
            p_svc: None,
        };

        instance.create_and_set_proxy(Some(namespace_path))?;

        Ok(instance)
    }

    /// Like `with_namespace_path`, but assumes that COM is managed externally.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it is the caller's responsibility to ensure that COM is initialized and will not be uninitialized before the connection object is dropped.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// let _initialized_com = COMLibrary::new()?;
    ///
    /// // Later, in the same thread.
    /// let wmi_con = unsafe { WMIConnection::with_initialized_com(Some("ROOT\\CIMV2"))? };
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn with_initialized_com(namespace_path: Option<&str>) -> Result<Self, WMIError> {
        let mut instance = Self {
            _com_con: None,
            p_loc: None,
            p_svc: None,
        };

        instance.create_and_set_proxy(namespace_path)?;

        Ok(instance)
    }

    pub fn svc(&self) -> *mut IWbemServices {
        self.p_svc.unwrap().as_ptr()
    }

    fn loc(&self) -> *mut IWbemLocator {
        self.p_loc.unwrap().as_ptr()
    }

    fn create_locator(&mut self) -> Result<(), WMIError> {
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

        self.p_loc = NonNull::new(p_loc as *mut IWbemLocator);

        debug!("Got locator {:?}", self.p_loc);

        Ok(())
    }

    fn create_services(&mut self, path: &str) -> Result<(), WMIError> {
        debug!("Calling ConnectServer");

        let mut p_svc = ptr::null_mut::<IWbemServices>();

        let object_path_bstr = BStr::from_str(path)?;

        unsafe {
            check_hres((*self.loc()).ConnectServer(
                object_path_bstr.as_bstr(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut p_svc,
            ))?;
        }

        self.p_svc = NonNull::new(p_svc as *mut IWbemServices);

        debug!("Got service {:?}", self.p_svc);

        Ok(())
    }

    fn set_proxy(&self) -> Result<(), WMIError> {
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
