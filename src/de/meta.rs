use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::forward_to_deserialize_any;

/// Return the fields of a struct.
/// Taken directly from https://github.com/serde-rs/serde/issues/1110
///
pub fn struct_name_and_fields<'de, T>(
) -> Result<(&'static str, &'static [&'static str]), serde::de::value::Error>
where
    T: Deserialize<'de>,
{
    struct StructNameAndFieldsDeserializer<'a> {
        name: &'a mut Option<&'static str>,
        fields: &'a mut Option<&'static [&'static str]>,
    }

    impl<'de, 'a> Deserializer<'de> for StructNameAndFieldsDeserializer<'a> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(de::Error::custom("I'm just here for the fields"))
        }

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

        fn deserialize_struct<V>(
            self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.name = Some(name);
            *self.fields = Some(fields);
            self.deserialize_any(visitor)
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct seq tuple
            tuple_struct map enum identifier ignored_any
        }
    }

    let mut name = None;
    let mut fields = None;

    let _ = T::deserialize(StructNameAndFieldsDeserializer {
        name: &mut name,
        fields: &mut fields,
    });

    match name {
        None =>  Err(de::Error::custom("Expected a named struct. \
            Hint: You cannot use a HashMap<...> in this context because it requires the struct to have a name")),
        Some(name) => {
            Ok((name, fields.unwrap()))
        }
    }
}

mod tests {
    use super::*;
    use crate::Variant;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            Caption: String,
            Name: String,
        }

        let (name, fields) = struct_name_and_fields::<Win32_OperatingSystem>().unwrap();

        assert_eq!(name, "Win32_OperatingSystem");
        assert_eq!(fields, ["Caption", "Name"]);
    }

    #[test]
    fn it_works_with_rename() {
        #[derive(Deserialize, Debug)]
        #[serde(rename = "Win32_OperatingSystem")]
        #[serde(rename_all = "PascalCase")]
        struct Win32OperatingSystem {
            caption: String,
            name: String,
        }

        let (name, fields) = struct_name_and_fields::<Win32OperatingSystem>().unwrap();

        assert_eq!(name, "Win32_OperatingSystem");
        assert_eq!(fields, ["Caption", "Name"]);
    }

    #[test]
    fn it_fails_for_non_structs() {
        let err = struct_name_and_fields::<HashMap<String, Variant>>().unwrap_err();

        assert!(format!("{:?}", err).contains("Expected a named struct"));
    }
}
