# wmi
[Documentation](https://ohadravid.github.io/wmi-rs/docs/wmi/)

WMI crate for rust.
Currently in beta.

```toml
# Cargo.toml
[dependencies]
wmi = "0.2"
```


## Examples

Queries can be deserialized info a free-form `HashMap` or a `struct`:

```rust
use std::collections::HashMap;
use serde::Deserialize;

use wmi::{from_wbem_class_obj, COMLibrary, Variant, WMIConnection, WMIDateTime};

let com_con = COMLibrary::new().unwrap();
let wmi_con = WMIConnection::new(com_con.into()).unwrap();

let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem").unwrap();

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

let results: Vec<Win32_OperatingSystem> = wmi_con.query().unwrap();

for os in results {
    println!("{:#?}", os);
}
```

## License
 
The `wmi` crate is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.