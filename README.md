# wmi
WMI crate for rust.
Currently 🚧 WIP 🚧.

## Examples

Queries can be deserialized info a free-form `HashMap` or a `struct`:

```rust
use std::collections::HashMap;
use serde::Deserialize;

use wmi::{from_wbem_class_obj, COMLibrary, Variant, WMIConnection, WMIDateTime};

let com_con = COMLibrary::new().unwrap();
let wmi_con = WMIConnection::new(com_con.into()).unwrap();

let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem")?;

for os_res in enumerator {
    let os: HashMap<String, Variant> = from_wbem_class_obj(&os_res.unwrap())?;

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

let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem")?;

for os_res in enumerator {
    let os: Win32_OperatingSystem = from_wbem_class_obj(&os_res.unwrap())?;

    println!("{:#?}", os);
}
```


## License
 
The `wmi` crate is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.