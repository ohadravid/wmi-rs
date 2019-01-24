use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::forward_to_deserialize_any;

/// Return the fields of a struct.
/// Taken directly from https://github.com/serde-rs/serde/issues/1110
///
fn struct_name_and_fields<'de, T>() -> (&'static str, &'static [&'static str])
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
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map enum identifier ignored_any
        }
    }

    let mut name = None;
    let mut fields = None;

    let _ = T::deserialize(StructNameAndFieldsDeserializer {
        name: &mut name,
        fields: &mut fields,
    });

    (name.unwrap(), fields.unwrap())
}

mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Win32_OperatingSystem {
        Caption: String,
        Name: String,
    }

    #[test]
    fn it_works() {
        let (name, fields) = struct_name_and_fields::<Win32_OperatingSystem>();

        assert_eq!(name, "Win32_OperatingSystem");
        assert_eq!(fields, ["Caption", "Name"]);
    }
}
