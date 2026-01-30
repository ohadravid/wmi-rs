use crate::{IWbemClassWrapper, context::WMIContext, utils::WMIResult};
use log::debug;
use std::marker::PhantomData;
use windows::{
    Win32::{
        Foundation::{CO_E_NOTINITIALIZED, RPC_E_TOO_LATE},
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, CoCreateInstance, CoIncrementMTAUsage, CoInitializeSecurity,
                CoSetProxyBlanket, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL, RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE,
            },
            Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE},
            Wmi::{
                IWbemContext, IWbemLocator, IWbemServices, WBEM_FLAG_CONNECT_USE_MAX_WAIT,
                WBEM_FLAG_RETURN_WBEM_COMPLETE, WbemLocator,
            },
        },
    },
    core::BSTR,
};

fn init_security() -> windows_core::Result<()> {
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

/// ```compile_fail
/// let wmi = wmi::WMIConnection::new().unwrap();
/// _test_not_send(wmi);
/// ```
fn _test_not_send(_s: impl Send) {}

/// A connection to the local WMI provider.
///
/// <div class="warning">
///
/// If COM is uninitialized when a new WMI connection is created, it will be initialized (using [`CoIncrementMTAUsage`]),
/// and the [default security policy] will be set. COM will NOT be uninitialized when the connection is dropped.
///
/// If this is not what you want, then you must initialize COM yourself. See [Hosting the Windows Runtime] for more.
///
/// </div>
///
/// [`CoIncrementMTAUsage`]: https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coincrementmtausage
/// [default security policy]: https://learn.microsoft.com/en-us/windows/win32/wmisdk/setting-the-default-process-security-level-using-c-
/// [Hosting the Windows Runtime]: https://kennykerrca.wordpress.com/2018/03/24/cppwinrt-hosting-the-windows-runtime/
#[derive(Clone, Debug)]
pub struct WMIConnection {
    // Force the type to be `!Send`, as each thread must initialize COM and a separate connection.
    _phantom: PhantomData<*mut ()>,
    pub(crate) svc: IWbemServices,
    pub(crate) ctx: WMIContext,
}

impl WMIConnection {
    /// Creates a connection with a default `CIMV2` namespace path.
    pub fn new() -> WMIResult<Self> {
        Self::with_namespace_path("ROOT\\CIMV2")
    }

    /// Creates a connection with the given namespace path.
    ///
    /// ```edition2018
    /// # fn main() -> wmi::WMIResult<()> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// let wmi_con = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_namespace_path(namespace_path: &str) -> WMIResult<Self> {
        let loc = create_locator_or_init()?;

        let ctx = WMIContext::new()?;
        let svc = create_services(&loc, namespace_path, None, None, None, &ctx.0)?;

        let this = Self {
            _phantom: PhantomData,
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
    /// See [`IWbemLocator::ConnectServer`](https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemlocator-connectserver).
    ///
    /// # Example
    /// ```no_run
    /// # use wmi::*;
    /// # fn main() -> WMIResult<()> {
    /// let wmi_con = WMIConnection::with_credentials(
    ///     "ServerName",         // Server name or IP address
    ///     Some("username"),
    ///     Some("password"),
    ///     Some("domain"),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_credentials(
        server: &str,
        username: Option<&str>,
        password: Option<&str>,
        domain: Option<&str>,
    ) -> WMIResult<Self> {
        Self::with_credentials_and_namespace(server, "ROOT\\CIMV2", username, password, domain)
    }

    /// Creates a connection to a remote computer with the given namespace path and credentials.
    ///
    /// # Example
    /// ```no_run
    /// # use wmi::*;
    /// # fn main() -> WMIResult<()> {
    /// let wmi_con = WMIConnection::with_credentials_and_namespace(
    ///     "ServerName",         // Server name or IP address
    ///     "ROOT\\CIMV2",        // Namespace path
    ///     Some("username"),
    ///     Some("password"),
    ///     Some("domain"),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_credentials_and_namespace(
        server: &str,
        namespace_path: &str,
        username: Option<&str>,
        password: Option<&str>,
        domain: Option<&str>,
    ) -> WMIResult<Self> {
        let loc = create_locator_or_init()?;

        // Build the full namespace path for remote connection
        let full_namespace = &format!(r"\\{}\{}", server, namespace_path);

        let ctx = WMIContext::new()?;
        let svc = create_services(&loc, full_namespace, username, password, domain, &ctx.0)?;

        let this = Self {
            _phantom: PhantomData,
            svc,
            ctx,
        };

        this.set_proxy_for_remote()?;
        Ok(this)
    }

    /// Create an instance of existing class
    ///
    /// # Example
    /// ```no_run
    /// # use wmi::*;
    /// # fn main() -> WMIResult<()> {
    /// let wmi_con = WMIConnection::with_namespace_path("root\\standardcimv2")?;
    /// let rule_class = wmi_con.get_object("MSFT_NetFirewallHyperVRule")?;
    /// let instance = rule_class.spawn_instance()?;
    /// instance.put_property("ElementName", "Blocking outbound rule")?;
    /// instance.put_property("InstanceID", "{ed7dee72-7ca3-4728-ad16-e6ee5c465c98}")?;
    /// instance.put_property("Action", 4)?;
    /// instance.put_property("Enabled", 1)?;
    /// instance.put_property("Direction", 2)?;
    /// wmi_con.put_instance(&instance)?;
    /// # Ok(())
    /// # }
    /// ````
    pub fn put_instance(&self, instance: &IWbemClassWrapper) -> WMIResult<()> {
        unsafe {
            self.svc
                .PutInstance(&instance.inner, WBEM_FLAG_RETURN_WBEM_COMPLETE, None, None)?;
        }
        Ok(())
    }

    /// Delete an instance at path
    ///
    /// # Example
    /// ```no_run
    /// # use wmi::*;
    /// # fn main() -> WMIResult<()> {
    /// let wmi_con = WMIConnection::with_namespace_path("root\\standardcimv2")?;
    /// let rule_path = r#"MSFT_NetFirewallHyperVRule.InstanceID="{ed7dee72-7ca3-4728-ad16-e6ee5c465c98}""#;
    /// wmi_con.delete_instance(rule_path)?;
    /// # Ok(())
    /// # }
    /// ````
    pub fn delete_instance(&self, path: &str) -> WMIResult<()> {
        let path = BSTR::from(path);
        unsafe {
            self.svc
                .DeleteInstance(&path, WBEM_FLAG_RETURN_WBEM_COMPLETE, None, None)?
        };
        Ok(())
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

fn create_locator() -> windows_core::Result<IWbemLocator> {
    debug!("Calling CoCreateInstance for CLSID_WbemLocator");

    let loc = unsafe { CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)? };

    debug!("Got locator {:?}", loc);

    Ok(loc)
}

fn create_locator_or_init() -> windows_core::Result<IWbemLocator> {
    let loc_res = create_locator();
    match loc_res {
        // If COM is not initialized, initialize it and try again.
        // Based on [`load_factory`](https://github.com/microsoft/windows-rs/blob/945130accc25ac18a47054115e861ca704a37eb5/crates/libs/core/src/imp/factory_cache.rs#L73)
        // from the `windows-rs` crate.
        Err(err) if err.code() == CO_E_NOTINITIALIZED => {
            let _ = unsafe { CoIncrementMTAUsage() }?;
            let sec_result = init_security();

            // If security was initialized already, there's no need to return an error.
            if let Err(err) = &sec_result
                && err.code() != RPC_E_TOO_LATE
            {
                sec_result?;
            }

            create_locator()
        }
        loc_res => loc_res,
    }
}

fn create_services(
    loc: &IWbemLocator,
    namespace_path: &str,
    username: Option<&str>,
    password: Option<&str>,
    authority: Option<&str>,
    ctx: &IWbemContext,
) -> WMIResult<IWbemServices> {
    let namespace_path = BSTR::from(namespace_path);
    let user = BSTR::from(username.unwrap_or_default());
    let password = BSTR::from(password.unwrap_or_default());
    let authority = BSTR::from(authority.unwrap_or_default());

    let svc = unsafe {
        loc.ConnectServer(
            &namespace_path,
            &user,
            &password,
            &BSTR::new(),
            WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
            &authority,
            ctx,
        )?
    };

    Ok(svc)
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use rusty_fork::rusty_fork_test;

    use super::*;

    #[test]
    fn it_can_create_multiple_connections() {
        {
            let _ = WMIConnection::new();
        }
        {
            let _ = WMIConnection::new();
        }
    }

    #[test]
    fn it_can_connect_to_localhost_without_credentials() {
        // Connect to localhost with empty credentials
        let result = WMIConnection::with_credentials("localhost", None, None, None);

        // The connection should succeed
        assert!(
            result.is_ok(),
            "Failed to connect to localhost without credentials: {:?}",
            result.err()
        );
    }

    rusty_fork_test! {
        /// See https://github.com/ohadravid/wmi-rs/issues/136.
        #[test]
        fn it_can_run_as_thread_local_in_non_main_thread() {
            use crate::WMIConnection;

            thread_local! {
                static WMI: Option<WMIConnection> = {
                    let wmi = WMIConnection::new().unwrap();

                    Some(wmi)
                };
            }

            let thread = std::thread::spawn(|| {
                WMI.with(|_wmi| {
                    assert!(true);
                })
            });

            thread.join().unwrap();
        }
    }
}
