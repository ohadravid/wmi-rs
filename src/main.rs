#![feature(ptr_internals)]

use failure::{Error};
use log::{debug, info, Level};

use wmi::connection::{COMLibrary, WMIConnection};
use wmi::variant::Variant;
use std::collections::HashMap;
use wmi::de::wbem_class_de::from_wbem_class_obj;

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

    Ok(())
}
