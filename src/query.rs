use crate::from_wbem_class_obj;
use crate::{
    connection::WMIConnection,
    consts::{WBEM_FLAG_ALWAYS, WBEM_FLAG_NONSYSTEM_ONLY},
    de::meta::struct_name_and_fields,
    safearray::{get_string_array, SafeArrayDestroy},
    utils::check_hres,
};
use failure::Error;
use log::debug;
use serde::de;
use std::collections::HashMap;
use std::{ptr, ptr::Unique};
use widestring::WideCString;
use winapi::{
    shared::ntdef::NULL,
    um::{
        oaidl::SAFEARRAY,
        wbemcli::{IEnumWbemClassObject, IWbemClassObject},
        wbemcli::{WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE},
    },
};

pub enum FilterValue {
    Bool(bool),
    Number(i64),
    Str(&'static str),
    String(String),
}

fn build_query<'de, T>(filters: Option<&HashMap<String, FilterValue>>) -> String
where
    T: de::Deserialize<'de>,
{
    let (name, fields) = struct_name_and_fields::<T>();

    let optional_where_clause = match filters {
        None => String::new(),
        Some(filters) => {
            if filters.is_empty() {
                String::new()
            } else {
                let mut conditions = vec![];

                for (field, filter) in filters {
                    let value = match filter {
                        FilterValue::Bool(b) => {
                            if *b {
                                "true".to_owned()
                            } else {
                                "false".to_owned()
                            }
                        }
                        FilterValue::Number(n) => format!("{}", n),
                        FilterValue::Str(s) => format!("\"{}\"", s),
                        FilterValue::String(s) => format!("\"{}\"", s),
                    };

                    conditions.push(format!("{} = {}", field, value));
                }

                // Just to make testing easier.
                conditions.sort();

                format!("WHERE {}", conditions.join(" AND "))
            }
        }
    };

    let query_text = format!(
        "SELECT {} FROM {} {}",
        fields.join(","),
        name,
        optional_where_clause
    );

    query_text
}

pub struct QueryResultEnumerator<'a> {
    wmi_con: &'a WMIConnection,
    p_enumerator: Option<Unique<IEnumWbemClassObject>>,
}

impl WMIConnection {
    /// Execute the given query and return an iterator of WMI pointers.
    /// It's better to use the other query methods, since this is relatively low level.
    pub fn exec_query_native_wrapper(&self, query: impl AsRef<str>) -> Result<QueryResultEnumerator, Error> {
        let query_language = WideCString::from_str("WQL")?;
        let query = WideCString::from_str(query)?;

        let mut p_enumerator = NULL as *mut IEnumWbemClassObject;

        unsafe {
            check_hres((*self.svc()).ExecQuery(
                query_language.as_ptr() as *mut _,
                query.as_ptr() as *mut _,
                (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY) as i32,
                ptr::null_mut(),
                &mut p_enumerator,
            ))?;
        }

        debug!("Got enumerator {:?}", p_enumerator);

        Ok(QueryResultEnumerator {
            wmi_con: self,
            p_enumerator: Unique::new(p_enumerator),
        })
    }

    /// Execute a free-text query and deserialize the results.
    /// Can be used either with a struct (like `query` and `filtered_query`),
    /// but also with a generic map.
    ///
    /// ```edition2018
    /// #
    /// con.raw_query::<HashMap<String, Variant>>("SELECT Name FROM Win32_OperatingSystem");
    /// #
    pub fn raw_query<T>(&self, query: impl AsRef<str>) -> Result<Vec<T>, Error>
        where
            T: de::DeserializeOwned,
    {
        let enumerator = self.exec_query_native_wrapper(query)?;

        enumerator
            .map(|item| match item {
                Ok(wbem_class_obj) => {
                    let value = from_wbem_class_obj(&wbem_class_obj);

                    value.map_err(Error::from)
                }
                Err(e) => Err(e),
            })
            .collect()
    }

    /// Query all the objects of type T.
    ///
    /// ```edition2018
    /// #
    /// struct Win32_OperatingSystem {
    ///     Name: String,
    /// }
    /// con.query::<Win32_OperatingSystem>();
    /// #
    pub fn query<T>(&self) -> Result<Vec<T>, Error>
    where
        T: de::DeserializeOwned,
    {
        let query_text = build_query::<T>(None);

        self.raw_query(&query_text)
    }

    /// Query all the objects of type T, while filtering according to `filters`.
    pub fn filtered_query<T>(&self, filters: &HashMap<String, FilterValue>) -> Result<Vec<T>, Error>
    where
        T: de::DeserializeOwned,
    {
        let query_text = build_query::<T>(Some(&filters));

        self.raw_query(&query_text)
    }

}

impl<'a> QueryResultEnumerator<'a> {
    pub fn p(&self) -> *mut IEnumWbemClassObject {
        self.p_enumerator.unwrap().as_ptr()
    }
}

impl<'a> Drop for QueryResultEnumerator<'a> {
    fn drop(&mut self) {
        if let Some(p_enumerator) = self.p_enumerator {
            unsafe {
                (*p_enumerator.as_ptr()).Release();
            }
        }
    }
}

/// A wrapper around a raw pointer to IWbemClassObject, which also takes care of releasing
/// the object when dropped.
///
#[derive(Debug)]
pub struct IWbemClassWrapper {
    pub inner: Option<Unique<IWbemClassObject>>,
}

impl IWbemClassWrapper {
    pub fn new(ptr: Option<Unique<IWbemClassObject>>) -> Self {
        Self { inner: ptr }
    }

    /// Return the names of all the properties of the given object.
    ///
    pub fn list_properties(&self) -> Result<Vec<String>, Error> {
        // This will store the properties names from the GetNames call.
        let mut p_names = NULL as *mut SAFEARRAY;

        let ptr = self.inner.unwrap().as_ptr();

        unsafe {
            check_hres((*ptr).GetNames(
                ptr::null(),
                WBEM_FLAG_ALWAYS | WBEM_FLAG_NONSYSTEM_ONLY,
                ptr::null_mut(),
                &mut p_names,
            ))
        }?;

        let res = get_string_array(p_names);

        unsafe {
            check_hres(SafeArrayDestroy(p_names))?;
        }

        res
    }
}

impl Drop for IWbemClassWrapper {
    fn drop(&mut self) {
        if let Some(pcls_obj) = self.inner {
            let ptr = pcls_obj.as_ptr();

            unsafe {
                (*ptr).Release();
            }
        }
    }
}

impl<'a> Iterator for QueryResultEnumerator<'a> {
    type Item = Result<IWbemClassWrapper, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pcls_obj = NULL as *mut IWbemClassObject;
        let mut return_value = 0;

        if self.p_enumerator.is_none() {
            return None;
        }

        let raw_enumerator_prt = self.p_enumerator.unwrap().as_ptr();

        let res = unsafe {
            check_hres((*raw_enumerator_prt).Next(
                WBEM_INFINITE as i32,
                1,
                &mut pcls_obj,
                &mut return_value,
            ))
        };

        if let Err(e) = res {
            return Some(Err(e.into()));
        }

        if return_value == 0 {
            return None;
        }

        debug!(
            "Got enumerator {:?} and obj {:?}",
            self.p_enumerator, pcls_obj
        );

        let pcls_wrapper = IWbemClassWrapper::new(Unique::new(pcls_obj));

        Some(Ok(pcls_wrapper))
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod tests {
    use super::*;
    use crate::connection::COMLibrary;
    use crate::connection::WMIConnection;
    use crate::datetime::WMIDateTime;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::collections::hash_map::RandomState;

    #[test]
    fn it_works() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        let enumerator = wmi_con
            .exec_query_native_wrapper("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();
            let mut props = w.list_properties().unwrap();

            props.sort();

            assert_eq!(props.len(), 64);
            assert_eq!(props[..2], ["BootDevice", "BuildNumber"]);
            assert_eq!(props[props.len() - 2..], ["Version", "WindowsDirectory"])
        }
    }

    #[test]
    fn it_can_query_a_struct() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
        }

        let results = wmi_con.query::<Win32_OperatingSystem>().unwrap();

        for os in results {
            assert_eq!(os.Caption, "Microsoft Windows 10 Pro");
        }
    }

    #[test]
    fn it_builds_correct_query_without_filters() {
        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
        }

        let query = build_query::<Win32_OperatingSystem>(None);
        let select_part = r#"SELECT Caption FROM Win32_OperatingSystem "#.to_owned();

        assert_eq!(query, select_part);
    }


    #[test]
    fn it_builds_correct_query() {
        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
        }

        let mut filters = HashMap::new();

        filters.insert("C1".to_string(), FilterValue::Str("a"));
        filters.insert("C2".to_string(), FilterValue::String("b".to_string()));
        filters.insert("C3".to_string(), FilterValue::Number(42));
        filters.insert("C4".to_string(), FilterValue::Bool(false));

        let query = build_query::<Win32_OperatingSystem>(Some(&filters));
        let select_part = r#"SELECT Caption FROM Win32_OperatingSystem "#.to_owned();
        let where_part = r#"WHERE C1 = "a" AND C2 = "b" AND C3 = 42 AND C4 = false"#;

        assert_eq!(query, select_part + where_part);
    }

    #[test]
    fn it_can_filter() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        #[derive(Deserialize, Debug)]
        struct Win32_Process {
            Name: String,
        }

        let mut filters = HashMap::new();

        filters.insert("Name".to_owned(), FilterValue::Str("cargo.exe"));

        let results = wmi_con.filtered_query::<Win32_Process>(&filters).unwrap();

        assert!(results.len() >= 1);

        for proc in results {
            assert_eq!(proc.Name, "cargo.exe");
        }
    }
}
