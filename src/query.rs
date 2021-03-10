use crate::BStr;
use crate::de::wbem_class_de::from_wbem_class_obj;
use crate::result_enumerator::{IWbemClassWrapper, QueryResultEnumerator};
use crate::{
    connection::WMIConnection, de::meta::struct_name_and_fields, utils::check_hres, WMIError,
};
use log::trace;
use serde::de;
use std::collections::HashMap;
use std::ptr;
use std::ptr::NonNull;
use winapi::um::wbemcli::IWbemClassObject;
use winapi::{
    shared::ntdef::NULL,
    um::{
        wbemcli::IEnumWbemClassObject,
        wbemcli::{
            WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_FLAG_RETURN_WBEM_COMPLETE,
        },
    },
};

pub enum FilterValue {
    Bool(bool),
    Number(i64),
    Str(&'static str),
    String(String),
}

impl From<String> for FilterValue {
    fn from(value: String) -> Self {
        FilterValue::String(value)
    }
}

impl From<&'static str> for FilterValue {
    fn from(value: &'static str) -> Self {
        FilterValue::Str(value)
    }
}

impl From<i64> for FilterValue {
    fn from(value: i64) -> Self {
        FilterValue::Number(value)
    }
}

impl From<bool> for FilterValue {
    fn from(value: bool) -> Self {
        FilterValue::Bool(value)
    }
}

/// Build an SQL query for the given filters, over the given type (using it's name and fields).
/// For example, for:
///
/// # Examples
///
/// For a struct such as:
///
/// ```edition2018
/// # use wmi::*;
/// # use serde::Deserialize;
/// #[derive(Deserialize, Debug)]
/// #[serde(rename = "Win32_OperatingSystem")]
/// #[serde(rename_all = "PascalCase")]
/// struct OperatingSystem {
///     caption: String,
///     debug: bool,
/// }
/// ```
///
/// The resulting query (with no filters) will look like:
/// ```
/// "SELECT Caption, Debug FROM Win32_OperatingSystem";
/// ```
///
pub fn build_query<'de, T>(filters: Option<&HashMap<String, FilterValue>>) -> Result<String, WMIError>
where
    T: de::Deserialize<'de>,
{
    let (name, fields) = struct_name_and_fields::<T>()?;

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
                        FilterValue::Str(s) => quote_and_escape_wql_str(s),
                        FilterValue::String(s) => quote_and_escape_wql_str(&s),
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

    Ok(query_text)
}

/// Quote/escape a string for WQL.
///
/// [2.2.1 WQL Query] references [DMTF-DSP0004] ("CIM") which, in reading section "4.11.1 String Constants",
/// seems to only require that `\` and `"` be escaped.  It's underspecified in CIM what happens with unicode
/// values - perhaps `\xNNNN` escaping would be appropriate for a more general purpose CIM string escaping
/// function - but in my testing on Windows 10.0.19041.572, it seems that such values do *not* need to be
/// escaped for WQL, and are treated as their expected unicode values just fine.
///
/// [2.2.1 WQL Query]:  https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-wmi/6c8a38f4-4ee1-47cb-99f1-b42718a575ce
/// [DMTF-DSP0004]:     https://www.dmtf.org/sites/default/files/standards/documents/DSP0004V2.3_final.pdf
///
/// ```edition2018
/// # use wmi::query::quote_and_escape_wql_str;
/// assert_eq!(quote_and_escape_wql_str(r#"C:\Path\With"In Name"#), r#""C:\\Path\\With\"In Name""#);
/// ```
pub fn quote_and_escape_wql_str(s: &str) -> String {
    let mut o = String::with_capacity(s.as_bytes().len() + 2);
    o.push('"');
    for ch in s.chars() {
        match ch {
            '\"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            ch   => o.push(ch),
        }
    }
    o.push('"');
    o
}

impl WMIConnection {
    /// Execute the given query and return an iterator of WMI pointers.
    /// It's better to use the other query methods, since this is relatively low level.
    ///
    pub fn exec_query_native_wrapper(
        &self,
        query: impl AsRef<str>,
    ) -> Result<QueryResultEnumerator, WMIError> {
        let query_language = BStr::from_str("WQL")?;
        let query = BStr::from_str(query.as_ref())?;

        let mut p_enumerator = NULL as *mut IEnumWbemClassObject;

        unsafe {
            check_hres((*self.svc()).ExecQuery(
                query_language.as_bstr(),
                query.as_bstr(),
                (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY) as i32,
                ptr::null_mut(),
                &mut p_enumerator,
            ))?;
        }

        trace!("Got enumerator {:?}", p_enumerator);

        Ok(unsafe { QueryResultEnumerator::new(self, p_enumerator) })
    }

    /// Execute a free-text query and deserialize the results.
    /// Can be used either with a struct (like `query` and `filtered_query`),
    /// but also with a generic map.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use std::collections::HashMap;
    /// # use wmi::*;
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    /// let results: Vec<HashMap<String, Variant>> = con.raw_query("SELECT Name FROM Win32_OperatingSystem")?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn raw_query<T>(&self, query: impl AsRef<str>) -> Result<Vec<T>, WMIError>
    where
        T: de::DeserializeOwned,
    {
        let enumerator = self.exec_query_native_wrapper(query)?;

        enumerator
            .map(|item| match item {
                Ok(wbem_class_obj) => wbem_class_obj.into_desr(),
                Err(e) => Err(e),
            })
            .collect()
    }

    /// Query all the objects of type T.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// use wmi::*;
    /// use serde::Deserialize;
    ///
    /// let con = WMIConnection::new(COMLibrary::new()?.into())?;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_Process {
    ///     Name: String,
    /// }
    ///
    /// let procs: Vec<Win32_Process> = con.query()?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn query<T>(&self) -> Result<Vec<T>, WMIError>
    where
        T: de::DeserializeOwned,
    {
        let query_text = build_query::<T>(None)?;

        self.raw_query(&query_text)
    }

    /// Query all the objects of type T, while filtering according to `filters`.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use std::collections::HashMap;
    /// # use wmi::*;
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    /// use serde::Deserialize;
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_Process {
    ///     Name: String,
    /// }
    ///
    /// let mut filters = HashMap::new();
    ///
    /// filters.insert("Name".to_owned(), FilterValue::Str("cargo.exe"));
    ///
    /// let results = con.filtered_query::<Win32_Process>(&filters).unwrap();
    ///
    /// assert!(results.len() >= 1);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn filtered_query<T>(
        &self,
        filters: &HashMap<String, FilterValue>,
    ) -> Result<Vec<T>, WMIError>
    where
        T: de::DeserializeOwned,
    {
        let query_text = build_query::<T>(Some(&filters))?;

        self.raw_query(&query_text)
    }

    /// Get a single object of type T.
    /// If none are found, an error is returned.
    /// If more than one object is found, all but the first are ignored.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    /// # use wmi::*;
    /// use serde::Deserialize;
    /// #[derive(Deserialize)]
    /// struct Win32_OperatingSystem {
    ///     Name: String,
    /// }
    ///
    /// let os = con.get::<Win32_OperatingSystem>()?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get<T>(&self) -> Result<T, WMIError>
    where
        T: de::DeserializeOwned,
    {
        let results = self.query()?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| WMIError::ResultEmpty)
    }

    /// Get a WMI object by path, and return a wrapper around a WMI pointer.
    /// It's better to use the `get_by_path` method, since this function is more low level.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    /// let raw_os = con.get_raw_by_path(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#)?;
    /// assert_eq!(raw_os.class()?, "Win32_OperatingSystem");
    ///
    /// #[derive(Deserialize)]
    /// struct Win32_OperatingSystem {
    ///     Name: String,
    /// }
    ///
    /// let os: Win32_OperatingSystem = raw_os.into_desr()?;
    /// println!("{}", os.Name);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_raw_by_path(
        &self,
        object_path: impl AsRef<str>,
    ) -> Result<IWbemClassWrapper, WMIError> {
        let object_path = BStr::from_str(object_path.as_ref())?;

        let mut pcls_obj = NULL as *mut IWbemClassObject;

        unsafe {
            check_hres((*self.svc()).GetObject(
                object_path.as_bstr(),
                WBEM_FLAG_RETURN_WBEM_COMPLETE as i32,
                ptr::null_mut(),
                &mut pcls_obj,
                ptr::null_mut(),
            ))?;
        }

        let pcls_ptr = NonNull::new(pcls_obj).ok_or(WMIError::NullPointerResult)?;

        let pcls_wrapper = unsafe { IWbemClassWrapper::new(pcls_ptr) };

        Ok(pcls_wrapper)
    }

    /// Get a WMI object by path, and return a deserialized object.
    /// This is useful when the type of the object at the path in known at compile time.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    /// #[derive(Deserialize)]
    /// struct Win32_OperatingSystem {
    ///     Name: String,
    /// }
    /// let os = con.get_by_path::<Win32_OperatingSystem>(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#)?;
    /// #   Ok(())
    /// # }
    /// ```
    ///
    /// It's possible to have a path where the type of the WMI object is not known at compile time.
    /// Either use `get_raw_by_path` and the `.class()` to find out the real type of the object,
    /// or if the object is only of a few possible types, deserialize it to an enum:
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use std::collections::HashMap;
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Win32_Group {
    ///     __Path: String,
    /// }
    ///
    /// let mut filters = HashMap::new();
    /// filters.insert("Name".into(), "Administrators".into());
    ///
    ///
    /// let admin_group: Win32_Group = con
    ///     .filtered_query(&filters)?
    ///     .into_iter()
    ///     .next()
    ///     .unwrap();
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Win32_Account {
    ///     __Path: String,
    /// }
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Win32_UserAccount {
    ///     Caption: String,
    /// }
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Win32_SystemAccount {
    ///     Caption: String,
    /// }
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Win32_GroupUser{ }
    ///
    /// // Groups contain multiple types of objects, all inheriting from `Win32_Account`.
    /// let accounts_in_group: Vec<Win32_Account> = con.associators::<_, Win32_GroupUser>(&admin_group.__Path)?;
    ///
    /// #[derive(Deserialize, Debug)]
    /// enum User {
    ///     #[serde(rename = "Win32_SystemAccount")]
    ///     System(Win32_SystemAccount),
    ///     #[serde(rename = "Win32_UserAccount")]
    ///     User(Win32_UserAccount),
    /// };
    ///
    /// for account in accounts_in_group {
    ///     let user: User = con.get_by_path(&account.__Path)?;
    ///     println!("{:?}", user);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_by_path<T>(&self, object_path: &str) -> Result<T, WMIError>
    where
        T: de::DeserializeOwned,
    {
        let wbem_class_obj = self.get_raw_by_path(object_path)?;

        from_wbem_class_obj(&wbem_class_obj)
    }

    /// Query all the associators of type T of the given object.
    /// The `object_path` argument can be provided by querying an object wih it's `__Path` property.
    /// `AssocClass` must be have the name as the conneting association class between the original object and the results.
    /// See <https://docs.microsoft.com/en-us/windows/desktop/cimwin32prov/win32-diskdrivetodiskpartition> for example.
    ///
    /// ```edition2018
    /// # fn main() -> Result<(), wmi::WMIError> {
    /// # use wmi::*;
    /// # use serde::Deserialize;
    /// # let con = WMIConnection::new(COMLibrary::new()?.into())?;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_DiskDrive {
    ///     // `__Path` is a WMI-internal property used to uniquely identify objects.
    ///     __Path: String,
    ///     Caption: String,
    /// }
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_DiskPartition {
    ///     Caption: String,
    /// }
    ///
    /// // There's no need to specify any fields here, since only the name of the struct is used by `associators`.
    /// #[derive(Deserialize, Debug)]
    /// struct Win32_DiskDriveToDiskPartition {}
    ///
    /// let disk = con.get::<Win32_DiskDrive>()?;
    /// let results = con.associators::<Win32_DiskPartition, Win32_DiskDriveToDiskPartition>(&disk.__Path)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn associators<ResultClass, AssocClass>(
        &self,
        object_path: &str,
    ) -> Result<Vec<ResultClass>, WMIError>
    where
        ResultClass: de::DeserializeOwned,
        AssocClass: de::DeserializeOwned,
    {
        let (class_name, _fields) = struct_name_and_fields::<ResultClass>()?;
        let (association_class, _) = struct_name_and_fields::<AssocClass>()?;

        // See more at:
        // https://docs.microsoft.com/en-us/windows/desktop/wmisdk/associators-of-statement
        let query = format!(
            "ASSOCIATORS OF {{{object_path}}} WHERE AssocClass = {association_class} ResultClass = {class_name}",
            object_path = object_path,
            association_class = association_class,
            class_name = class_name
        );

        self.raw_query(&query)
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    use crate::tests::fixtures::*;
    use crate::{Variant, WMIError};
    use winapi::shared::ntdef::HRESULT;
    use winapi::um::wbemcli::WBEM_E_INVALID_QUERY;

    #[test]
    fn it_works() {
        let wmi_con = wmi_con();

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
    fn it_fails_gracefully() {
        let wmi_con = wmi_con();

        let enumerator = wmi_con
            .exec_query_native_wrapper("SELECT NoSuchField FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            assert!(res.is_err())
        }
    }

    #[test]
    fn it_fails_gracefully_with_invalid_sql() {
        let wmi_con = wmi_con();

        let enumerator = wmi_con.exec_query_native_wrapper("42").unwrap();

        // Show how to detect which error had occurred.
        for res in enumerator {
            match res {
                Ok(_) => assert!(false),
                Err(wmi_err) => match wmi_err {
                    WMIError::HResultError { hres } => {
                        assert_eq!(hres, WBEM_E_INVALID_QUERY as HRESULT);
                    }
                    _ => assert!(false),
                },
            }
        }
    }

    #[test]
    fn it_can_query_a_struct() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
        }

        let results = wmi_con.query::<Win32_OperatingSystem>().unwrap();

        for os in results {
            assert!(os.Caption.starts_with("Microsoft Windows"));
        }
    }

    #[test]
    fn it_can_query_a_hashmap() {
        let wmi_con = wmi_con();

        let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query("SELECT Name FROM Win32_OperatingSystem").unwrap();

        let results_as_json = serde_json::to_string(&results).unwrap();
        assert!(results_as_json.starts_with(r#"[{"Name":"Microsoft Windows"#));
    }

    #[test]
    fn it_fails_gracefully_when_querying_a_struct() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            NoSuchField: String,
        }

        let result = wmi_con.query::<Win32_OperatingSystem>();

        assert!(result.is_err());
    }

    #[test]
    fn it_builds_correct_query_without_filters() {
        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
        }

        let query = build_query::<Win32_OperatingSystem>(None).unwrap();
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
        filters.insert("C5".to_string(), FilterValue::String(r#"with " and \ chars"#.to_string()));

        let query = build_query::<Win32_OperatingSystem>(Some(&filters)).unwrap();
        let select_part = r#"SELECT Caption FROM Win32_OperatingSystem "#.to_owned();
        let where_part = r#"WHERE C1 = "a" AND C2 = "b" AND C3 = 42 AND C4 = false AND C5 = "with \" and \\ chars""#;

        assert_eq!(query, select_part + where_part);
    }

    #[test]
    fn it_can_filter() {
        let wmi_con = wmi_con();

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

    #[test]
    fn it_can_query_all_classes() {
        let wmi_con = wmi_con();
        let classes = [
            "Win32_Service",
            "Win32_Process",
            "Win32_OperatingSystem",
            "Win32_TimeZone",
            "Win32_ComputerSystem",
            "Win32_NetworkAdapter",
            "Win32_NetworkAdapterConfiguration",
            "Win32_LogicalDisk",
            "Win32_PhysicalMemory",
            "Win32_StartupCommand",
            "Win32_NetworkLoginProfile",
            "Win32_Share",
            "Win32_MappedLogicalDisk",
            "Win32_DiskDrive",
            "Win32_Product",
            "Win32_IP4RouteTable",
            "Win32_NetworkConnection",
            "Win32_Group",
            // Only works under 64bit.
            // "Win32_ShadowCopy",
        ];

        for class_name in classes.iter() {
            let results: Vec<HashMap<String, Variant>> = wmi_con
                .raw_query(format!("SELECT * FROM {}", class_name))
                .unwrap();

            for res in results {
                match res.get("Caption") {
                    Some(Variant::String(_)) | Some(Variant::Null) => assert!(true),
                    _ => assert!(false),
                }
            }
        }

        // Associators. TODO: Support this in the desr logic (so a Disk can have `Partitions`).
        let associators_classes = [
            "Win32_DiskDriveToDiskPartition",
            "Win32_LogicalDiskToPartition",
        ];

        for class_name in associators_classes.iter() {
            let results: Vec<HashMap<String, Variant>> = wmi_con
                .raw_query(format!("SELECT * FROM {}", class_name))
                .unwrap();

            for res in results {
                match res.get("Antecedent") {
                    Some(Variant::String(s)) => assert!(s != ""),
                    _ => assert!(false),
                }
            }
        }

        let results: Vec<HashMap<String, Variant>> =
            wmi_con.raw_query("SELECT * FROM Win32_GroupUser").unwrap();

        for res in results {
            match res.get("GroupComponent") {
                Some(Variant::String(s)) => assert!(s != ""),
                _ => assert!(false),
            }

            match res.get("PartComponent") {
                Some(Variant::String(s)) => assert!(s != ""),
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn con_get_return_a_single_object() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_Process {
            Name: String,
        }

        let proc = wmi_con.get::<Win32_Process>().unwrap();

        assert_ne!(proc.Name, "");
    }

    #[test]
    fn con_error_for_query_without_struct() {
        let wmi_con = wmi_con();

        let res: Result<Vec<HashMap<String, Variant>>, _> = wmi_con.query();

        assert!(res.is_err());
    }

    #[test]
    fn it_can_query_associators() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_DiskDrive {
            __Path: String,
            Caption: String,
        }

        #[derive(Deserialize, Debug)]
        struct Win32_DiskPartition {
            Caption: String,
        }

        #[derive(Deserialize, Debug)]
        struct Win32_DiskDriveToDiskPartition {}

        let disk = wmi_con.get::<Win32_DiskDrive>().unwrap();

        let results = wmi_con
            .associators::<Win32_DiskPartition, Win32_DiskDriveToDiskPartition>(&disk.__Path)
            .unwrap();

        assert!(results.len() >= 1);

        for part in results {
            assert!(part.Caption.contains("Partition #"));
        }
    }

    #[test]
    fn it_can_query_floats() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_ReliabilityStabilityMetrics {
            SystemStabilityIndex: f64,
        }

        let metric = wmi_con.get::<Win32_ReliabilityStabilityMetrics>().unwrap();
        assert!(metric.SystemStabilityIndex >= 0.0);

        #[derive(Deserialize, Debug)]
        struct Win32_WinSAT {
            CPUScore: f32,
        }

        let sat = wmi_con.get::<Win32_WinSAT>().unwrap();
        assert!(sat.CPUScore >= 0.0);
    }

    #[test]
    fn it_can_query_slashes_and_unicode() {
        let tmp_dir = tempdir::TempDir::new("PlayStation™Now").unwrap();
        let wmi_con = wmi_con();
        let tmp_dir_path = tmp_dir.path().to_string_lossy().to_string();
        
        #[derive(Deserialize, Debug)]
        struct Win32_Directory {
            Name: String,
        }

        let mut filters = HashMap::new();
        filters.insert(String::from("Name"), FilterValue::String(tmp_dir_path.clone()));
        let directory = wmi_con.filtered_query::<Win32_Directory>(&filters).unwrap().pop().unwrap();
        assert_eq!(directory.Name.to_lowercase(), tmp_dir_path.to_lowercase());
    }

    #[test]
    fn con_get_return_an_object_by_path() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_Process {
            __Path: String,
            Name: String,
            ProcessId: i64,
        }

        let procs = wmi_con.query::<Win32_Process>().unwrap();

        let proc = &procs[3];

        let proc_by_path = wmi_con.get_by_path::<Win32_Process>(&proc.__Path).unwrap();

        assert_eq!(&proc_by_path, proc);

        let proc_by_path_hashmap: HashMap<String, Variant> =
            wmi_con.get_by_path(&proc.__Path).unwrap();

        assert_eq!(
            proc_by_path_hashmap.get("ProcessId").unwrap(),
            &Variant::I8(proc.ProcessId)
        );
    }

    #[test]
    fn con_get_return_an_object_by_path_from_actual_path() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_OperatingSystem {
            Caption: String,
        }

        let os = wmi_con
            .get_by_path::<Win32_OperatingSystem>(r#"\\.\root\cimv2:Win32_OperatingSystem=@"#)
            .unwrap();

        assert!(os.Caption.contains("Microsoft Windows"));
    }

    #[test]
    fn con_get_return_a_raw_object_by_path_from_actual_path() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_Account {
            __Path: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_Group {
            __Path: String,
            Caption: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_UserAccount {
            Caption: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_SystemAccount {
            Caption: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct Win32_GroupUser {}

        let mut filters = HashMap::new();
        filters.insert("Name".into(), "Administrators".into());

        let group: Win32_Group = wmi_con
            .filtered_query(&filters)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        let accounts_in_group: Vec<Win32_Account> = wmi_con
            .associators::<_, Win32_GroupUser>(&group.__Path)
            .unwrap();

        #[derive(Deserialize, Debug)]
        enum User {
            #[serde(rename = "Win32_SystemAccount")]
            System(Win32_SystemAccount),
            #[serde(rename = "Win32_UserAccount")]
            User(Win32_UserAccount),
        }

        for account in accounts_in_group {
            let raw_account = wmi_con.get_raw_by_path(&account.__Path).unwrap();

            // Completely dynamic.
            match raw_account.class().unwrap().as_str() {
                "Win32_UserAccount" | "Win32_SystemAccount" | "Win32_Group" => {
                    // OK.
                }
                _ => panic!(),
            };

            let _account_as_hashmap: HashMap<String, Variant> = raw_account.into_desr().unwrap();

            // Enum based desr.
            let _raw_account: User = wmi_con.get_by_path(&account.__Path).unwrap();
        }
    }
}
