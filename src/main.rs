#![feature(ptr_internals)]
use failure::Error;
use log::{debug, info, Level};
use serde::Deserialize;

use std::collections::HashMap;
use wmi::{from_wbem_class_obj, COMLibrary, Variant, WMIConnection, WMIDateTime};

fn main() -> Result<(), Error> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    debug!("Starting up");

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let results: Vec<HashMap<String, Variant>> =
        wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem")?;

    for os in results {
        info!("{:#?}", os);
    }

    #[derive(Deserialize, Debug)]
    struct Win32_OperatingSystem {
        Caption: String,
        Name: String,
        CurrentTimeZone: i16,
        Debug: bool,
        EncryptionLevel: u32,
        ForegroundApplicationBoost: u8,
        LastBootUpTime: WMIDateTime,
    }

    let results: Vec<Win32_OperatingSystem> =
        wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem")?;

    for os in results {
        println!("{:#?}", os);
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_OperatingSystem")]
    #[serde(rename_all = "PascalCase")]
    struct OperatingSystem {
        caption: String,
        debug: bool,
    }

    let results: Vec<OperatingSystem> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }

    Ok(())
}
