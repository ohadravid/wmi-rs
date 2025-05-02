use crate::context::WMIContext;
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
    RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE,
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
/// See: <https://github.com/microsoft/windows-rs/issues/1169#issuecomment-926877227>.
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

/// A connection to the local WMI provider.
///
#[derive(Clone, Debug)]
pub struct WMIConnection {
    _com_con: COMLibrary,
    pub svc: IWbemServices,
    pub(crate) ctx: WMIContext,
}

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
        let svc = create_services(&loc, namespace_path, None, None, None)?;
        let ctx = WMIContext::new()?;

        let this = Self {
            _com_con: com_lib,
            svc,
            ctx,
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

    /// Creates a connection to a remote computer with a default `CIMV2` namespace path.
    /// https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemlocator-connectserver
    ///
    /// # Example
    /// ```no_run
    /// # use wmi::*;
    /// # fn main() -> WMIResult<()> {
    /// let com_lib = COMLibrary::new()?;
    /// let wmi_con = WMIConnection::with_credentials(
    ///     "ServerName",         // Server name or IP address
    ///     "username",
    ///     "password",
    ///     "domain",
    ///     com_lib
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_credentials(
        server: &str,
        username: &str,
        password: &str,
        domain: &str,
        com_lib: COMLibrary,
    ) -> WMIResult<Self> {
        Self::with_credentials_and_namespace(
            server,
            "ROOT\\CIMV2",
            username,
            password,
            domain,
            com_lib,
        )
    }

    /// Creates a connection to a remote computer with the given namespace path and credentials.
    ///
    /// # Example
    /// ```no_run
    /// # use wmi::*;
    /// # fn main() -> WMIResult<()> {
    /// let com_lib = COMLibrary::new()?;
    /// let wmi_con = WMIConnection::with_credentials_and_namespace(
    ///     "ServerName",         // Server name or IP address
    ///     "ROOT\\CIMV2",        // Namespace path
    ///     "username",
    ///     "password",
    ///     "domain",
    ///     com_lib
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_credentials_and_namespace(
        server: &str,
        namespace_path: &str,
        username: &str,
        password: &str,
        domain: &str,
        com_lib: COMLibrary,
    ) -> WMIResult<Self> {
        let loc = create_locator()?;

        // Build the full namespace path for remote connection
        let full_namespace = &format!(r"\\{}\{}", server, namespace_path);

        let svc = create_services(
            &loc,
            full_namespace,
            Some(username),
            Some(password),
            Some(domain),
        )?;
        let ctx = WMIContext::new()?;

        let this = Self {
            _com_con: com_lib,
            svc,
            ctx,
        };

        this.set_proxy_for_remote()?;
        Ok(this)
    }

    // Additional authentication for remote WMI connections
    fn set_proxy_for_remote(&self) -> WMIResult<()> {
        debug!("Calling CoSetProxyBlanket for remote connection");

        unsafe {
            CoSetProxyBlanket(
                &self.svc,
                RPC_C_AUTHN_WINNT,             // RPC_C_AUTHN_xxx
                RPC_C_AUTHZ_NONE,              // RPC_C_AUTHZ_xxx
                None,                          // Server principal name
                RPC_C_AUTHN_LEVEL_PKT_PRIVACY, // Stronger authentication level for remote
                RPC_C_IMP_LEVEL_IMPERSONATE,   // Impersonation level
                None,                          // Client identity
                EOAC_NONE,                     // Capability flags
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

fn create_services(
    loc: &IWbemLocator,
    namespace_path: &str,
    username: Option<&str>,
    password: Option<&str>,
    authority: Option<&str>,
) -> WMIResult<IWbemServices> {
    let namespace_bstr = BSTR::from(namespace_path);

    // Create BSTRs for credentials only if they are provided
    let user_bstr = match username {
        Some(user) => BSTR::from(user),
        None => BSTR::new(),
    };

    let pass_bstr = match password {
        Some(pass) => BSTR::from(pass),
        None => BSTR::new(),
    };

    let authority_bstr = match authority {
        Some(auth) => BSTR::from(auth),
        None => BSTR::new(),
    };

    let svc = unsafe {
        loc.ConnectServer(
            &namespace_bstr,
            &user_bstr,
            &pass_bstr,
            &BSTR::new(),
            WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
            &authority_bstr,
            None,
        )?
    };

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

    #[test]
    fn it_can_connect_to_localhost_without_credentials() {
        let com_lib = COMLibrary::new().unwrap();

        // Connect to localhost with empty credentials
        let result = WMIConnection::with_credentials("localhost", "", "", "", com_lib);

        // The connection should succeed
        assert!(
            result.is_ok(),
            "Failed to connect to localhost without credentials: {:?}",
            result.err()
        );
    }
}
