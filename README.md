# wmi
WMI crate for rust.
Currently 🚧 WIP 🚧.

## Example

(Query is hard-coded to return the `Name` property. Told you it's WIP).

```rust
let com_lib = COMLibrary::new().unwrap();
let wmi_con = WMIConnection::new(com_lib.into()).unwrap();

let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem").unwrap();

for name in enumerator {
    println!("I am {}", name.unwrap());
}
```


## License
 
The `wmi` crate is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.