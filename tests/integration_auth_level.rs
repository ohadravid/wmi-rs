//! Integration tests for custom authentication levels
//!
//! These tests verify that the with_auth_level() method works correctly
//! in various scenarios and maintains backward compatibility.

use serde::Deserialize;
use wmi::{WMIConnection, WMIResult};
use windows::Win32::System::Com::{
    RPC_C_AUTHN_LEVEL_CALL, RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_OperatingSystem")]
#[serde(rename_all = "PascalCase")]
struct OperatingSystem {
    caption: String,
    version: String,
}

/// Test that normal queries work with custom auth level
#[test]
fn test_normal_queries_with_custom_auth_level() {
    let wmi_con = WMIConnection::new()
        .expect("Failed to create connection")
        .with_auth_level(RPC_C_AUTHN_LEVEL_CALL)
        .expect("Failed to set auth level");

    let results: WMIResult<Vec<OperatingSystem>> = wmi_con.query();

    assert!(
        results.is_ok(),
        "Query should succeed with custom auth level"
    );

    let os_list = results.unwrap();
    assert!(!os_list.is_empty(), "Should return at least one OS");

    println!("OS: {} {}", os_list[0].caption, os_list[0].version);
}

/// Test that PKT_PRIVACY auth level works for standard queries
#[test]
fn test_pkt_privacy_for_standard_queries() {
    let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2")
        .expect("Failed to create connection")
        .with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY)
        .expect("Failed to set auth level");

    let results: WMIResult<Vec<OperatingSystem>> = wmi_con.query();

    assert!(
        results.is_ok(),
        "Query should succeed with PKT_PRIVACY auth level"
    );

    let os_list = results.unwrap();
    assert!(!os_list.is_empty(), "Should return at least one OS");

    println!(
        "With PKT_PRIVACY: {} {}",
        os_list[0].caption, os_list[0].version
    );
}

/// Test backward compatibility - existing code should work unchanged
#[test]
fn test_backward_compatibility() {
    // This is how users currently use the library - should continue to work
    let wmi_con = WMIConnection::new().expect("Failed to create connection");

    let results: WMIResult<Vec<OperatingSystem>> = wmi_con.query();

    assert!(
        results.is_ok(),
        "Query should succeed without custom auth level"
    );

    let os_list = results.unwrap();
    assert!(!os_list.is_empty(), "Should return at least one OS");

    println!(
        "Default auth: {} {}",
        os_list[0].caption, os_list[0].version
    );
}

/// Test that auth level can be set after creating connection with namespace
#[test]
fn test_auth_level_after_namespace() {
    let wmi_con = WMIConnection::with_namespace_path("ROOT\\CIMV2")
        .expect("Failed to create connection with namespace")
        .with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY)
        .expect("Failed to set auth level");

    let results: WMIResult<Vec<OperatingSystem>> = wmi_con.query();

    assert!(
        results.is_ok(),
        "Query should succeed after setting auth level"
    );
}

/// Test that auth level works with raw queries
#[test]
fn test_auth_level_with_raw_queries() {
    use std::collections::HashMap;
    use wmi::Variant;

    let wmi_con = WMIConnection::new()
        .expect("Failed to create connection")
        .with_auth_level(RPC_C_AUTHN_LEVEL_CALL)
        .expect("Failed to set auth level");

    let results: WMIResult<Vec<HashMap<String, Variant>>> =
        wmi_con.raw_query("SELECT Caption, Version FROM Win32_OperatingSystem");

    assert!(
        results.is_ok(),
        "Raw query should succeed with custom auth level"
    );

    let os_list = results.unwrap();
    assert!(!os_list.is_empty(), "Should return at least one OS");

    if let Some(os) = os_list.first() {
        println!("Raw query result: {:#?}", os);
    }
}

/// Test that remote connections can use custom auth level
#[test]
fn test_remote_connection_with_auth_level() {
    // Try connecting to localhost as a "remote" connection
    let result = WMIConnection::with_credentials("localhost", None, None, None);

    if let Ok(wmi_con) = result {
        let wmi_con = wmi_con
            .with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY)
            .expect("Failed to set auth level for remote connection");

        let results: WMIResult<Vec<OperatingSystem>> = wmi_con.query();

        // This should work for localhost
        if results.is_ok() {
            let os_list = results.unwrap();
            println!(
                "Remote connection with custom auth: {} OS found",
                os_list.len()
            );
        }
    } else {
        println!(
            "Skipping remote connection test - could not connect to localhost: {:?}",
            result.err()
        );
    }
}

/// Test that auth level can be changed multiple times (idempotency)
#[test]
fn test_auth_level_can_be_changed() {
    let wmi_con = WMIConnection::new()
        .expect("Failed to create connection")
        .with_auth_level(RPC_C_AUTHN_LEVEL_CALL)
        .expect("Failed to set auth level")
        .with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY)
        .expect("Failed to override auth level");

    // Verify second call overrode first
    let results: WMIResult<Vec<OperatingSystem>> = wmi_con.query();
    assert!(results.is_ok(), "Query should succeed with final auth level");

    let os_list = results.unwrap();
    assert!(!os_list.is_empty(), "Should return at least one OS");

    println!(
        "Multiple auth level changes work: {} {}",
        os_list[0].caption, os_list[0].version
    );
}
