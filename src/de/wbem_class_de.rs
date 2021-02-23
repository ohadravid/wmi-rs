use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, Unexpected,
    VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

use std::iter::Peekable;

use crate::result_enumerator::IWbemClassWrapper;
use crate::WMIError;

pub struct Deserializer<'a> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    pub wbem_class_obj: &'a IWbemClassWrapper,
}

impl<'a> Deserializer<'a> {
    pub fn from_wbem_class_obj(wbem_class_obj: &'a IWbemClassWrapper) -> Self {
        Deserializer { wbem_class_obj }
    }
}

pub fn from_wbem_class_obj<T>(wbem_class_obj: &IWbemClassWrapper) -> Result<T, WMIError>
where
    T: DeserializeOwned,
{
    let mut deserializer = Deserializer::from_wbem_class_obj(wbem_class_obj);
    let t = T::deserialize(&mut deserializer)?;

    Ok(t)
}

struct WMIEnum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> WMIEnum<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for WMIEnum<'a, 'de> {
    type Error = WMIError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for WMIEnum<'a, 'de> {
    type Error = WMIError;

    // Newtype variants can be deserialized directly.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // All other possible enum variants are not supported.
    fn unit_variant(self) -> Result<(), Self::Error> {
        let unexp = Unexpected::UnitVariant;
        Err(de::Error::invalid_type(unexp, &"newtype variant"))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let unexp = Unexpected::TupleVariant;
        Err(de::Error::invalid_type(unexp, &"newtype variant"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let unexp = Unexpected::StructVariant;
        Err(de::Error::invalid_type(unexp, &"newtype variant"))
    }
}

struct WMIMapAccess<'a, 'de, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    fields: Peekable<I>,
    de: &'a Deserializer<'de>,
}

impl<'a, 'de, S, I> WMIMapAccess<'a, 'de, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    pub fn new(fields: I, de: &'a Deserializer<'de>) -> Self {
        Self {
            fields: fields.peekable(),
            de,
        }
    }
}

impl<'de, 'a, S, I> MapAccess<'de> for WMIMapAccess<'a, 'de, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    type Error = WMIError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(field) = self.fields.peek() {
            seed.deserialize(field.as_ref().into_deserializer())
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let current_field = self
            .fields
            .next()
            .ok_or_else(|| WMIError::SerdeError("Expected current field to not be None".into()))?;

        let property_value = self
            .de
            .wbem_class_obj
            .get_property(current_field.as_ref())?;

        seed.deserialize(property_value)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = WMIError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(WMIError::SerdeError(
            "Only structs and maps can be deserialized from WMI objects".into(),
        ))
    }

    fn deserialize_enum<V>(
        mut self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(WMIEnum::new(&mut self))
    }

    // When deserializing enums, return the object's class name as the expected enum variant.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let class_name = self.wbem_class_obj.class()?;
        visitor.visit_string(class_name)
    }

    // Support for deserializing `Wrapper(Win32_OperatingSystem)`.
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let fields = self.wbem_class_obj.list_properties()?;

        visitor.visit_map(WMIMapAccess::new(fields.iter(), &self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(WMIMapAccess::new(fields.iter(), &self))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct seq tuple
        tuple_struct ignored_any
    }
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::WMIDateTime;
    use crate::duration::WMIDuration;
    use crate::variant::Variant;
    use serde::Deserialize;
    use std::collections::HashMap;

    use crate::tests::fixtures::*;
    use std::process;

    #[test]
    fn it_works() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
            Name: String,
            CurrentTimeZone: i16,
            Debug: bool,

            // This actually returns as an i32 from COM.
            EncryptionLevel: u32,
            ForegroundApplicationBoost: u8,

            LastBootUpTime: WMIDateTime,
        }

        let enumerator = wmi_con
            .exec_query_native_wrapper("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();

            let w: Win32_OperatingSystem = from_wbem_class_obj(&w).unwrap();

            assert!(w.Caption.contains("Microsoft "));
            assert!(w.Name.contains("Microsoft ") && w.Name.contains("Partition"));
            assert_eq!(w.Debug, false);
            assert_eq!(w.EncryptionLevel, 256);
            assert_eq!(w.ForegroundApplicationBoost, 2);
            assert_eq!(
                w.LastBootUpTime.0.timezone().local_minus_utc() / 60,
                w.CurrentTimeZone as i32
            );
        }
    }

    #[test]
    fn it_desr_into_map() {
        let wmi_con = wmi_con();

        let enumerator = wmi_con
            .exec_query_native_wrapper("SELECT * FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();

            let w: HashMap<String, Variant> = from_wbem_class_obj(&w).unwrap();

            match w.get("Caption").unwrap() {
                Variant::String(s) => assert!(s.starts_with("Microsoft Windows")),
                _ => assert!(false),
            }

            assert_eq!(*w.get("Debug").unwrap(), Variant::Bool(false));

            let langs = w.get("MUILanguages").unwrap();

            match langs {
                Variant::Array(langs) => {
                    assert!(langs.contains(&Variant::String("en-US".into())));
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn it_desr_into_map_with_selected_fields() {
        let wmi_con = wmi_con();

        let enumerator = wmi_con
            .exec_query_native_wrapper("SELECT Caption FROM Win32_OperatingSystem")
            .unwrap();

        for res in enumerator {
            let w = res.unwrap();

            let w: HashMap<String, Variant> = from_wbem_class_obj(&w).unwrap();

            match w.get("Caption").unwrap() {
                Variant::String(s) => assert!(s.starts_with("Microsoft Windows")),
                _ => assert!(false),
            }

            assert_eq!(w.get("Debug"), None);
        }
    }

    #[test]
    fn it_desr_array() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        struct Win32_ComputerSystem {
            BootStatus: Vec<i32>,
            Roles: Vec<String>,
        }

        let results: Vec<Win32_ComputerSystem> = wmi_con.query().unwrap();

        for res in results {
            assert_eq!(res.BootStatus.len(), 10);
            assert!(res.Roles.contains(&"NT".to_owned()));
        }
    }

    #[test]
    fn it_desr_option_string() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        pub struct Win32_Process {
            pub Name: String,
            pub CommandLine: Option<String>,
            pub ProcessID: u32,
        }

        let mut filters = HashMap::new();
        filters.insert("ProcessID".into(), 0.into());

        let system_proc: Win32_Process = wmi_con.filtered_query(&filters).unwrap().pop().unwrap();

        assert_eq!(system_proc.CommandLine, None);

        let mut filters = HashMap::new();
        filters.insert("ProcessID".into(), i64::from(process::id()).into());

        let current_proc: Win32_Process = wmi_con.filtered_query(&filters).unwrap().pop().unwrap();

        assert!(current_proc.CommandLine.is_some());
    }

    #[test]
    fn it_fail_to_desr_null_to_string() {
        // Values can return as Null / Empty from WMI.
        // It is impossible to `desr` such values to `String` (for example).
        // See `it_desr_option_string` on how to fix this error.
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        pub struct Win32_Process {
            pub Name: String,
            pub CommandLine: String,
            pub ProcessID: u32,
        }

        let mut filters = HashMap::new();
        filters.insert("ProcessID".into(), 0.into());

        let res: Result<Vec<Win32_Process>, _> = wmi_con.filtered_query(&filters);

        let err = res.err().unwrap();

        assert_eq!(
            format!("{}", err),
            "invalid type: Option value, expected a string"
        )
    }

    #[test]
    fn it_desr_duration() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        pub struct Win32_NetworkLoginProfile {
            pub PasswordAge: Option<WMIDuration>,
        }

        let _profiles: Vec<Win32_NetworkLoginProfile> = wmi_con.query().unwrap();
    }

    #[test]
    fn it_can_desr_newtype() {
        // Values can return as Null / Empty from WMI.
        // It is impossible to `desr` such values to `String` (for example).
        // See `it_desr_option_string` on how to fix this error.
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        pub struct Win32_Service {
            pub Name: String,
            pub PathName: Option<String>,
        }

        #[derive(Deserialize, Debug)]
        struct Wrapper(Win32_Service);

        let wrapped_service: Wrapper = wmi_con.get().unwrap();

        assert_ne!(&wrapped_service.0.Name, "")
    }

    #[test]
    fn it_can_desr_newtype_enum() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Debug)]
        pub struct Win32_UserAccount {
            pub __Path: String,
            pub Name: String,
        }

        #[derive(Deserialize, Debug)]
        pub struct Win32_SystemAccount {
            pub Name: String,
        }

        #[derive(Deserialize, Debug)]
        enum User {
            #[serde(rename = "Win32_SystemAccount")]
            System(Win32_SystemAccount),
            #[serde(rename = "Win32_UserAccount")]
            User(Win32_UserAccount),
        }

        let user: Win32_UserAccount = wmi_con.get().unwrap();

        let user_enum: User = wmi_con.get_by_path(&user.__Path).unwrap();

        match user_enum {
            User::System(_) => assert!(false),
            User::User(_) => assert!(true),
        };
    }
}
