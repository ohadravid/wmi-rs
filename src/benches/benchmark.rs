#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use criterion::{Criterion, criterion_group, criterion_main};
use wmi::{WMIConnection, WMIDateTime, Variant, COMLibrary};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Win32_Account")]
pub struct Account  {
    pub __Path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Win32_UserAccount")]
pub struct UserAccount {
    pub __Path: String,
    pub AccountType: i64,
    pub Caption: String,
    pub Description: String,
    pub Disabled: bool,
    pub Domain: String,
    pub FullName: String,
    pub LocalAccount: bool,
    pub Lockout: bool,
    pub Name: String,
    pub PasswordChangeable: bool,
    pub PasswordExpires: bool,
    pub PasswordRequired: bool,
    pub SID: String,
    pub SIDType: u64,
}

fn get_accounts(con: &WMIConnection) {
    let accounts: Vec<Account> = con.query().unwrap();
}

fn get_user_accounts(con: &WMIConnection) {
    let users: Vec<UserAccount> = con.query().unwrap();
}

fn get_user_accounts_hash_map(con: &WMIConnection) {
    let users: Vec<HashMap<String, Variant>> = con.raw_query("SELECT * FROM Win32_UserAccount").unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    // baseline: 41ms
    c.bench_function("get accounts", |b| {
        let wmi_con = WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap();
        b.iter(|| get_accounts(&wmi_con))
    });

    // baseline: 13ms
    c.bench_function("get user accounts", |b| {
        let wmi_con = WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap();
        b.iter(|| get_user_accounts(&wmi_con))
    });

    // baseline: 9ms
    c.bench_function("get user accounts with hashmap", |b| {
        let wmi_con = WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap();
        b.iter(|| get_user_accounts_hash_map(&wmi_con))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
