# wmi
[![Build Status](https://dev.azure.com/ohadrv/wmi-rs/_apis/build/status/ohadravid.wmi-rs?branchName=main)](https://dev.azure.com/ohadrv/wmi-rs/_build/latest?definitionId=1&branchName=main)
[![crates.io](https://img.shields.io/crates/v/wmi.svg)](https://crates.io/crates/wmi)
[![docs.rs](https://docs.rs/wmi/badge.svg)](https://docs.rs/crate/wmi)

WMI (Windows Management Instrumentation) crate for rust.

```toml
# Cargo.toml
[dependencies]
wmi = "0.8"
```


## Examples

Queries can be deserialized into a free-form `HashMap` or a `struct`:

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

## Async Queries

WMI supports async queries, with methods like [ExecAsyncQuery](https://docs.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execqueryasync).

This crate provides async methods under the `async-query` flag:

```toml
# Cargo.toml
[dependencies]
wmi = { version = "0.8", features = ["async-query"] }
```

The methods become available on `WMIConnection`

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use serde::Deserialize;
use wmi::{COMLibrary, Variant, WMIConnection, WMIDateTime};
use std::collections::HashMap;
use futures::executor::block_on;

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;
    
    block_on(exec_async_query(&wmi_con))?;
    
    Ok(())
}

async fn exec_async_query(wmi_con: &WMIConnection) -> Result<(), Box<dyn std::error::Error>> {
    let results: Vec<HashMap<String, Variant>> = 
        wmi_con.async_raw_query("SELECT * FROM Win32_OperatingSystem").await?;

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
    
    let results: Vec<Win32_OperatingSystem> = wmi_con.async_query().await?;
    
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
