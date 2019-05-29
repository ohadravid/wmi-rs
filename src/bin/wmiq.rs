use std::collections::HashMap;
use std::env::args;
use wmi::{COMLibrary, Variant, WMIConnection};

fn main() {
    let wmi_con = WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap();
    let args: Vec<String> = args().collect();
    let query = match args.get(1) {
        None => {
            println!("Expected an argument with a WMI query");
            return;
        }
        Some(query) => query,
    };

    let results: Vec<HashMap<String, Variant>> = match wmi_con.raw_query(&query) {
        Err(e) => {
            println!("Couldn't run query {} because of {:?}", query, e);
            return;
        }
        Ok(results) => results,
    };

    for (i, res) in results.iter().enumerate() {
        println!("Result {}", i);
        println!("{:#?}", res);
    }
}
