//! This module implements a custom serializer type, [`VariantStructSerializer`],
//! to serialize a Rust struct into a HashMap mapping field name strings to [`Variant`] values
use std::{any::type_name, fmt::Display};

use crate::{result_enumerator::IWbemClassWrapper, Variant, WMIConnection, WMIError};
use serde::{
    ser::{Impossible, SerializeStruct},
    Serialize, Serializer,
};
use thiserror::Error;

macro_rules! serialize_variant_err_stub {
    ($signature:ident, $type:ty) => {
        fn $signature(self, _v: $type) -> Result<Self::Ok, Self::Error> {
            Err(VariantSerializerError::UnsupportedVariantType(
                type_name::<$type>().to_string(),
            ))
        }
    };
}

macro_rules! serialize_variant {
    ($signature:ident, $type:ty) => {
        fn $signature(self, v: $type) -> Result<Self::Ok, Self::Error> {
            Ok(Variant::from(v))
        }
    };
}

pub(crate) struct VariantSerializer<'a> {
    pub(crate) wmi: &'a WMIConnection,
    pub(crate) instance: Option<IWbemClassWrapper>,
}

impl<'a> Serializer for VariantSerializer<'a> {
    type Ok = Variant;
    type Error = VariantSerializerError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = VariantInstanceSerializer<'a>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    serialize_variant!(serialize_bool, bool);
    serialize_variant!(serialize_i8, i8);
    serialize_variant!(serialize_i16, i16);
    serialize_variant!(serialize_i32, i32);
    serialize_variant!(serialize_i64, i64);
    serialize_variant!(serialize_u8, u8);
    serialize_variant!(serialize_u16, u16);
    serialize_variant!(serialize_u32, u32);
    serialize_variant!(serialize_u64, u64);
    serialize_variant!(serialize_f32, f32);
    serialize_variant!(serialize_f64, f64);

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // When starting from an instance, deserializing a unit means returning the original instance unmodified.
        Ok(self.instance.map(Variant::from).unwrap_or(Variant::Empty))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Variant::from(v.to_string()))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantSerializerError::UnsupportedVariantType(format!(
            "{variant}::{name}"
        )))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        let ser = self.serialize_struct(name, 0)?;

        ser.end()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Variant::from(variant.to_string()))
    }

    // Generic serializer code not relevant to this use case

    serialize_variant_err_stub!(serialize_char, char);
    serialize_variant_err_stub!(serialize_bytes, &[u8]);

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "None".to_string(),
        ))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(VariantSerializerError::UnsupportedVariantType(
            type_name::<T>().to_string(),
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "Sequence".to_string(),
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "Tuple".to_string(),
        ))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            name.to_string(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(format!(
            "{variant}::{name}"
        )))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(
            "Map".to_string(),
        ))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // We are only given an initialized instance when called from `exec_method`,
        // with the instance matching the method signature class.
        // Otherwise, we use the name of the struct to create. See test for  `Win32_Process` with "Create" and `Win32_ProcessStartup`.
        let instance = match self.instance {
            Some(instance) => instance,
            None => self.wmi.get_object(name)?.spawn_instance()?,
        };

        let ser = VariantInstanceSerializer {
            wmi: self.wmi,
            instance,
        };

        Ok(ser)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(VariantSerializerError::UnsupportedVariantType(format!(
            "{variant}::{name}"
        )))
    }
}

#[derive(Debug, Error)]
pub enum VariantSerializerError {
    #[error("Unknown error while serializing struct:\n{0}")]
    Unknown(String),
    #[error("{0} cannot be serialized to a Variant.")]
    UnsupportedVariantType(String),
    #[error("WMI error while serializing struct: \n {0}")]
    WMIError(#[from] WMIError),
}

impl serde::ser::Error for VariantSerializerError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        VariantSerializerError::Unknown(msg.to_string())
    }
}

pub(crate) struct VariantInstanceSerializer<'a> {
    instance: IWbemClassWrapper,
    wmi: &'a WMIConnection,
}

impl<'a> SerializeStruct for VariantInstanceSerializer<'a> {
    type Ok = Variant;

    type Error = VariantSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let variant = value.serialize(VariantSerializer {
            wmi: self.wmi,
            instance: None,
        })?;

        self.instance.put_property(key, variant)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Variant::Object(self.instance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::wmi_con;
    use serde::{Deserialize, Serialize};

    #[test]
    fn it_serialize_instance() {
        let wmi_con = wmi_con();

        #[derive(Deserialize)]
        struct StdRegProv;

        #[derive(Serialize)]
        struct GetBinaryValue {
            sSubKeyName: String,
            sValueName: String,
        }

        let in_params = GetBinaryValue {
            sSubKeyName: r#"SYSTEM\CurrentControlSet\Control\Windows"#.to_string(),
            sValueName: "FullProcessInformationSID".to_string(),
        };

        // Similar to how `exec_class_method` creates these objects.
        let method_instance = wmi_con
            .get_object("StdRegProv")
            .unwrap()
            .get_method("GetBinaryValue")
            .unwrap()
            .unwrap()
            .spawn_instance()
            .unwrap();

        let instance_from_ser = in_params
            .serialize(VariantSerializer {
                wmi: &wmi_con,
                instance: Some(method_instance),
            })
            .unwrap();

        let instance_from_ser = match instance_from_ser {
            Variant::Object(instance_from_ser) => instance_from_ser,
            _ => panic!("Unexpected value {:?}", instance_from_ser),
        };

        let expected_instance = wmi_con
            .get_object("StdRegProv")
            .unwrap()
            .get_method("GetBinaryValue")
            .unwrap()
            .unwrap()
            .spawn_instance()
            .unwrap();

        assert_eq!(
            instance_from_ser.class().unwrap(),
            expected_instance.class().unwrap()
        );

        assert_eq!(
            instance_from_ser.get_property("sSubKeyName").unwrap(),
            Variant::String(in_params.sSubKeyName)
        );
    }

    #[test]
    fn it_serialize_instance_nested() {
        let wmi_con = wmi_con();

        #[derive(Debug, Serialize, Default)]
        pub struct Win32_ProcessStartup {
            pub Title: String,
        }

        #[derive(Deserialize)]
        struct Win32_Process;

        #[derive(Serialize)]
        struct CreateInput {
            CommandLine: String,
            ProcessStartupInformation: Win32_ProcessStartup,
        }

        // Verify that `Win32_ProcessStartup` can be serialized.
        let startup_info = Win32_ProcessStartup {
            Title: "Pong".to_string(),
        };

        let startup_info_instance = startup_info
            .serialize(VariantSerializer {
                wmi: &wmi_con,
                instance: None,
            })
            .unwrap();

        let startup_info_instance = match startup_info_instance {
            Variant::Object(startup_info_instance) => startup_info_instance,
            _ => panic!("Unexpected value {:?}", startup_info_instance),
        };

        assert_eq!(
            startup_info_instance.class().unwrap(),
            "Win32_ProcessStartup"
        );
        assert_eq!(
            startup_info_instance.get_property("Title").unwrap(),
            Variant::String(startup_info.Title.clone())
        );

        let create_params = CreateInput {
            CommandLine: r#"ping -n 3 127.0.0.1"#.to_string(),
            ProcessStartupInformation: startup_info,
        };

        // Similar to how `exec_class_method` creates these objects.
        let method_instance = wmi_con
            .get_object("Win32_Process")
            .unwrap()
            .get_method("Create")
            .unwrap()
            .unwrap()
            .spawn_instance()
            .unwrap();

        let instance_from_ser = create_params
            .serialize(VariantSerializer {
                wmi: &wmi_con,
                instance: Some(method_instance),
            })
            .unwrap();

        let instance_from_ser = match instance_from_ser {
            Variant::Object(instance_from_ser) => instance_from_ser,
            _ => panic!("Unexpected value {:?}", instance_from_ser),
        };

        assert_eq!(
            instance_from_ser.get_property("CommandLine").unwrap(),
            Variant::String(create_params.CommandLine)
        );

        assert!(matches!(
            instance_from_ser
                .get_property("ProcessStartupInformation")
                .unwrap(),
            Variant::Object(_)
        ));
    }
}
