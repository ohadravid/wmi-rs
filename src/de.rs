use error::{Result, Error};
use serde::de::{
    self, Deserialize, DeserializeOwned, DeserializeSeed, Expected, IntoDeserializer, Unexpected,
    Visitor,
};
use std::ptr::Unique;


pub struct IWbemClassWrapper {
    inner: Option<Unique<IWbemClassObject>>,
}

impl IWbemClassWrapper {
    pub fn new(ptr: Option<Unique<IWbemClassObject>>) -> Self {
        Self {
            inner: ptr
        }
    }
}

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    wbem_class_obj: &'de IWbemClassWrapper,
}

impl<'de> Deserializer<'de> {
    pub fn from_wbem_class_obj(wbem_class_obj: &'de IWbemClassWrapper) -> Self {
        Deserializer { wbem_class_obj }
    }
}

pub fn from_wbem_class_obj<'a, T>(wbem_class_obj: &'de IWbemClassWrapper) -> Result<T>
    where
        T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_wbem_class_obj(wbem_class_obj);
    let t = T::deserialize(&mut deserializer)?;

    Ok(t)
}