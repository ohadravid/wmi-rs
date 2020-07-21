# wmi
[![Build Status](https://dev.azure.com/ohadrv/wmi-rs/_apis/build/status/ohadravid.wmi-rs?branchName=master)](https://dev.azure.com/ohadrv/wmi-rs/_build/latest?definitionId=1&branchName=master)
![crates.io](https://img.shields.io/crates/v/wmi.svg)

[Documentation](https://docs.rs/crate/wmi)

WMI (Windows Management Instrumentation) crate for rust.

```toml
# Cargo.toml
[dependencies]
wmi = "0.5"
```


## Examples

Queries can be deserialized info a free-form `HashMap` or a `struct`:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
    
use serde::Deserialize;
use wmi::{COMLibrary, Variant, WMIConnection, WMIDateTime};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;
    
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem")?;
    
    for os in results {
        println!("{:#?}", os);
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
    
    let results: Vec<Win32_OperatingSystem> = wmi_con.query()?;
    
    for os in results {
        println!("{:#?}", os);
    }
    
    Ok(())
}
```

## License
 
The `wmi` crate is licensed under either of
```text
Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
```
at your option.