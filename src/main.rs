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

    let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem")?;

    for os_res in enumerator {
        let os: HashMap<String, Variant> = from_wbem_class_obj(&os_res?)?;

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

    let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem")?;

    for os_res in enumerator {
        let os: Win32_OperatingSystem = from_wbem_class_obj(&os_res.unwrap())?;

        println!("{:#?}", os);
    }
    Ok(())
}
