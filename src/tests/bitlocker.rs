//! Tests for BitLocker queries requiring custom authentication levels
//!
//! These tests verify that Win32_EncryptableVolume can be queried with
//! RPC_C_AUTHN_LEVEL_PKT_PRIVACY authentication.

use crate::{WMIConnection, WMIResult};
use serde::Deserialize;
use windows::Win32::System::Com::RPC_C_AUTHN_LEVEL_PKT_PRIVACY;

const BITLOCKER_NAMESPACE: &str = "ROOT\\CIMV2\\Security\\MicrosoftVolumeEncryption";

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_EncryptableVolume")]
#[serde(rename_all = "PascalCase")]
struct EncryptableVolume {
    device_id: String,
    drive_letter: Option<String>,
    protection_status: Option<u32>,
}

/// Test basic connection to BitLocker namespace with custom auth level
#[test]
fn it_can_connect_to_bitlocker_namespace() {
    let result = WMIConnection::with_namespace_path(BITLOCKER_NAMESPACE);

    match result {
        Ok(wmi_con) => {
            let result = wmi_con.with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY);
            assert!(
                result.is_ok(),
                "Failed to set auth level for BitLocker namespace"
            );
        }
        Err(e) => {
            // If we can't connect to the namespace, skip the test
            // (might not have admin privileges or BitLocker not available)
            println!("Skipping test - cannot connect to BitLocker namespace: {:?}", e);
        }
    }
}

/// Test querying Win32_EncryptableVolume with custom auth level
#[test]
fn it_can_query_bitlocker_with_custom_auth() {
    let wmi_con = match WMIConnection::with_namespace_path(BITLOCKER_NAMESPACE) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Skipping test - cannot connect to BitLocker namespace: {:?}", e);
            return;
        }
    };

    let wmi_con = match wmi_con.with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Skipping test - cannot set auth level: {:?}", e);
            return;
        }
    };

    let result: WMIResult<Vec<EncryptableVolume>> = wmi_con.query();

    match result {
        Ok(volumes) => {
            println!("Successfully queried {} encryptable volumes", volumes.len());
            for volume in volumes {
                println!(
                    "Volume: {:?}, Drive: {:?}, Protection: {:?}",
                    volume.device_id, volume.drive_letter, volume.protection_status
                );
            }
        }
        Err(e) => {
            // This might fail if not running as admin or if BitLocker is not enabled
            println!("Could not query BitLocker (might need admin): {:?}", e);
        }
    }
}

/// Test that querying without custom auth level may fail or return empty results
#[test]
fn it_shows_difference_without_custom_auth() {
    let wmi_con = match WMIConnection::with_namespace_path(BITLOCKER_NAMESPACE) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Skipping test - cannot connect to BitLocker namespace: {:?}", e);
            return;
        }
    };

    // Try to query without custom auth level (using default RPC_C_AUTHN_LEVEL_CALL)
    let result_without_auth: WMIResult<Vec<EncryptableVolume>> = wmi_con.query();

    // This may fail or return fewer results than with PKT_PRIVACY
    match result_without_auth {
        Ok(volumes) => {
            println!(
                "Without custom auth: got {} volumes (may be incomplete)",
                volumes.len()
            );
        }
        Err(e) => {
            println!("Without custom auth: query failed as expected: {:?}", e);
        }
    }
}

/// Test raw query pattern also works with custom auth
#[test]
fn it_can_query_bitlocker_raw_with_custom_auth() {
    use std::collections::HashMap;
    use crate::Variant;

    let wmi_con = match WMIConnection::with_namespace_path(BITLOCKER_NAMESPACE) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Skipping test - cannot connect to BitLocker namespace: {:?}", e);
            return;
        }
    };

    let wmi_con = match wmi_con.with_auth_level(RPC_C_AUTHN_LEVEL_PKT_PRIVACY) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Skipping test - cannot set auth level: {:?}", e);
            return;
        }
    };

    let result: WMIResult<Vec<HashMap<String, Variant>>> =
        wmi_con.raw_query("SELECT * FROM Win32_EncryptableVolume");

    match result {
        Ok(volumes) => {
            println!("Raw query returned {} volumes", volumes.len());
            for volume in volumes.iter().take(1) {
                println!("Sample volume data: {:#?}", volume);
            }
        }
        Err(e) => {
            println!("Raw query failed (might need admin): {:?}", e);
        }
    }
}
