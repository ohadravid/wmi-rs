use crate::utils::WMIResult;
use crate::WMIError;
use log::debug;
use std::marker::PhantomData;
use windows::core::BSTR;
use windows::Win32::Foundation::RPC_E_TOO_LATE;
use windows::Win32::System::Com::{
    CoCreateInstance, CoSetProxyBlanket, CLSCTX_INPROC_SERVER, RPC_C_AUTHN_LEVEL_CALL,
};
use windows::Win32::System::Com::{
    CoInitializeEx, CoInitializeSecurity, COINIT_MULTITHREADED, EOAC_NONE,
    RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
use windows::Win32::System::Wmi::{
    IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_CONNECT_USE_MAX_WAIT,
};

/// A marker to indicate that the current thread was `CoInitialize`d.
///
/// # Note
///
/// `COMLibrary` should be treated as a singleton per thread:
///
/// ```edition2018
/// # use wmi::*;
/// thread_local! {
///     static COM_LIB: COMLibrary = COMLibrary::new().unwrap();
/// }
///
/// pub fn wmi_con() -> WMIConnection {
///     let com_lib = COM_LIB.with(|com| *com);
///     WMIConnection::new(com_lib).unwrap()
/// }
/// ```
#[derive(Clone, Copy, Debug)]
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

        match instance.init_security() {
            Ok(()) => {}
            // Security was already initialized, this is fine
            Err(WMIError::HResultError { hres }) if hres == RPC_E_TOO_LATE.0 => {}
            Err(err) => return Err(err),
        }

        Ok(instance)
    }

    /// `CoInitialize`s the COM library for use by the calling thread, but without setting the security context.
    ///
    pub fn without_security() -> WMIResult<Self> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? }

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

#[derive(Clone, Debug)]
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
    pub fn with_namespace_path(namespace_path: &str, com_lib: COMLibrary) -> WMIResult<Self> {
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
                RPC_C_AUTHN_WINNT, // RPC_C_AUTHN_xxx
                RPC_C_AUTHZ_NONE,  // RPC_C_AUTHZ_xxx
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

    let loc = unsafe { CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)? };

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

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_create_multiple_connections() {
        {
            let com_lib = COMLibrary::new().unwrap();
            let _ = WMIConnection::new(com_lib);
        }
        {
            let com_lib = COMLibrary::new().unwrap();
            let _ = WMIConnection::new(com_lib);
        }
    }
}
