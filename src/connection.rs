use crate::{utils::{WMIResult, check_hres}, BStr};
use winapi::{
    shared::{
        rpcdce::{
            RPC_C_AUTHN_LEVEL_CALL,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_AUTHN_WINNT,
            RPC_C_AUTHZ_NONE,
            RPC_C_IMP_LEVEL_IMPERSONATE,
        },
        ntdef::NULL,
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        wbemcli::{
            IWbemLocator,
            IWbemServices,
            CLSID_WbemLocator,
            IID_IWbemLocator,
            WBEM_FLAG_CONNECT_USE_MAX_WAIT,
        },
        combaseapi::{CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket},
        objbase::COINIT_MULTITHREADED,
        objidl::EOAC_NONE,
    },
};
use std::{marker::PhantomData, ptr::{self, NonNull}};
use log::debug;

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
        unsafe { check_hres(CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED))? }

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

/// ```compile_fail
/// let com = COMLibrary::new().unwrap();
/// _test_com_lib_not_send(com);
/// ```
fn _test_com_lib_not_send(_s: impl Send) {}

pub struct WMIConnection {
    _com_con: COMLibrary,
    p_loc: NonNull<IWbemLocator>,
    p_svc: NonNull<IWbemServices>,
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
        let p_loc = create_locator()?;
        let p_svc = match create_services(p_loc.as_ptr(), namespace_path) {
            Ok(v) => v,
            Err(err) => {
                unsafe { (*p_loc.as_ptr()).Release() };
                return Err(err);
            }
        };

        let this = Self {
            _com_con: com_lib,
            p_loc,
            p_svc,
        };

        this.set_proxy()?;
        Ok(this)
    }

    pub fn svc(&self) -> *mut IWbemServices {
        self.p_svc.as_ptr()
    }

    fn set_proxy(&self) -> WMIResult<()> {
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

fn create_locator() -> WMIResult<NonNull<IWbemLocator>> {
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

    let p_loc = NonNull::new(p_loc as *mut IWbemLocator).unwrap();

    debug!("Got locator {:?}", p_loc);

    Ok(p_loc)
}

fn create_services(loc: *const IWbemLocator, path: &str) -> WMIResult<NonNull<IWbemServices>> {
    debug!("Calling ConnectServer");

    let mut p_svc = ptr::null_mut::<IWbemServices>();

    let object_path_bstr = BStr::from_str(path)?;

    unsafe {
        check_hres((*loc).ConnectServer(
            object_path_bstr.as_bstr(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            WBEM_FLAG_CONNECT_USE_MAX_WAIT as _,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut p_svc,
        ))?;
    }

    let p_svc = NonNull::new(p_svc as *mut IWbemServices).unwrap();

    debug!("Got service {:?}", p_svc);

    Ok(p_svc)
}

impl Drop for WMIConnection {
    fn drop(&mut self) {
        unsafe {
            (*self.p_svc.as_ptr()).Release();
        }

        unsafe {
            (*self.p_loc.as_ptr()).Release();
        }
    }
}
