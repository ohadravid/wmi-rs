#![feature(ptr_internals)]

use failure::{format_err, Error};
use log::{debug, info, trace, Level};

use wmi::connection::{COMLibrary, WMIConnection};

fn main() -> Result<(), Error> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    debug!("Starting up");

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem")?;

    for name in enumerator {
        debug!("I am {}", name.unwrap());
    }

    Ok(())
}
