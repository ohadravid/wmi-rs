use crate::utils::WMIResult;
use std::marker::PhantomData;
use log::debug;
use windows::Win32::System::Wmi::{IWbemLocator, IWbemServices, WBEM_FLAG_CONNECT_USE_MAX_WAIT, WbemLocator};
use windows::Win32::System::Com::{CoSetProxyBlanket, CoCreateInstance, CLSCTX_INPROC_SERVER, RPC_C_AUTHN_LEVEL_CALL};
use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
use windows::Win32::System::Com::{
    CoInitializeEx, COINIT_MULTITHREADED, CoInitializeSecurity, RPC_C_AUTHN_LEVEL_DEFAULT,
    RPC_C_IMP_LEVEL_IMPERSONATE, EOAC_NONE
};
use windows::core::BSTR;

/// A marker to indicate that the current thread was `CoInitialize`d.
/// It can be freely copied within the same thread.
#[derive(Clone, Copy)]
pub struct COMLibrary {
    // Force the type to be `!Send`, as each thread must be initialized separately.
    _phantom: PhantomData<*mut ()>,
}

/// Initialize COM.
///
/// `CoUninitialize` will NOT be called when dropped.
/// See: https://github.com/microsoft/windows-rs/issues/1169#issuecomment-926877227
///
impl COMLibrary {
    /// `CoInitialize`s the COM library for use by the calling thread.
    ///
    pub fn new() -> WMIResult<Self> {
        let instance = Self::without_security()?;
        instance.init_security()?;

        Ok(instance)
    }

    /// `CoInitialize`s the COM library for use by the calling thread, but without setting the security context.
    ///
    pub fn without_security() -> WMIResult<Self> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED)? }

        let instance = Self {
            _phantom: PhantomData,
        };

        Ok(instance)
    }

    /// Assumes that COM was already initialized for this thread.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it is the caller's responsibility to ensure that COM is initialized
    /// and will not be uninitialized while any instance of object is in scope.
    ///
    /// ```edition2018
    /// # fn main() -> wmi::WMIResult<()> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// # let _actual_com = COMLibrary::new()?;
    /// let initialized_com = unsafe { COMLibrary::assume_initialized() };
    ///
    /// // Later, in the same thread.
    /// let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2", initialized_com)?;
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_initialized() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn init_security(&self) -> WMIResult<()> {
        unsafe {
            CoInitializeSecurity(
                    None,
                -1, // let COM choose.
                None,
                None,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            )?;
        };

        Ok(())
    }
}

/// ```compile_fail
/// let com = COMLibrary::new().unwrap();
/// _test_com_lib_not_send(com);
/// ```
fn _test_com_lib_not_send(_s: impl Send) {}

#[derive(Clone)]
pub struct WMIConnection {
    _com_con: COMLibrary,
    pub svc: IWbemServices,
}

/// A connection to the local WMI provider, which provides querying capabilities.
///
/// Currently does not support remote providers (e.g connecting to other computers).
///
impl WMIConnection {
    /// Creates a connection with a default `CIMV2` namespace path.
    pub fn new(com_lib: COMLibrary) -> WMIResult<Self> {
        Self::with_namespace_path("ROOT\\CIMV2", com_lib)
    }

    /// Creates a connection with the given namespace path.
    ///
    /// ```edition2018
    /// # fn main() -> wmi::WMIResult<()> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// let wmi_con = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", COMLibrary::new()?)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_namespace_path(
        namespace_path: &str,
        com_lib: COMLibrary,
    ) -> WMIResult<Self> {
        let loc = create_locator()?;
        let svc = create_services(&loc, namespace_path)?;

        let this = Self {
            _com_con: com_lib,
            svc,
        };

        this.set_proxy()?;
        Ok(this)
    }

    fn set_proxy(&self) -> WMIResult<()> {
        debug!("Calling CoSetProxyBlanket");

        unsafe {
            CoSetProxyBlanket(
                &self.svc,
                RPC_C_AUTHN_WINNT,           // RPC_C_AUTHN_xxx
                RPC_C_AUTHZ_NONE,            // RPC_C_AUTHZ_xxx
                None,
                RPC_C_AUTHN_LEVEL_CALL,      // RPC_C_AUTHN_LEVEL_xxx
                RPC_C_IMP_LEVEL_IMPERSONATE, // RPC_C_IMP_LEVEL_xxx
                None,                        // client identity
                EOAC_NONE,                   // proxy capabilities
            )?;
        }

        Ok(())
    }
}

fn create_locator() -> WMIResult<IWbemLocator> {
    debug!("Calling CoCreateInstance for CLSID_WbemLocator");

    let loc = unsafe {
        CoCreateInstance(
            &WbemLocator,
            None,
            CLSCTX_INPROC_SERVER,
        )?
    };

    debug!("Got locator {:?}", loc);

    Ok(loc)
}

fn create_services(loc: &IWbemLocator, path: &str) -> WMIResult<IWbemServices> {
    debug!("Calling ConnectServer");

    let object_path_bstr = BSTR::from(path);

    let svc = unsafe {
        loc.ConnectServer(
            &object_path_bstr,
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
            &BSTR::new(),
            None,
        )?
    };

    debug!("Got service {:?}", svc);

    Ok(svc)
}
