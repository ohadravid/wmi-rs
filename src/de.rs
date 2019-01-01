use failure::{Error};
use serde::de::{
    self, Deserialize, DeserializeOwned, DeserializeSeed, Expected, IntoDeserializer, Unexpected,
    Visitor,
};
use log::{debug, info};
use crate::query::IWbemClassWrapper;

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

pub fn from_wbem_class_obj<'a, T>(wbem_class_obj: &'a IWbemClassWrapper) -> Result<T, Error>
    where
        T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_wbem_class_obj(wbem_class_obj);
    let t = T::deserialize(&mut deserializer)?;

    Ok(t)
}


mod tests {
    use super::*;
    use crate::connection::COMLibrary;
    use crate::connection::WMIConnection;
    use serde::Deserialize;

    #[test]
    fn it_works() {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();

        let p_svc = wmi_con.svc();

        assert_eq!(p_svc.is_null(), false);

        #[derive(Deserialize, Debug)]
        struct Win32_OperatingSystem {
            pub Caption: String,
        }

        let enumerator = wmi_con.query("SELECT * FROM Win32_OperatingSystem").unwrap();

        for res in enumerator {
            let w = res.unwrap();

            let w: Win32_OperatingSystem = from_wbem_class_obj(&w).unwrap();

            debug!("I am {:?}", w);
            assert_eq!(w.Caption, "Microsoft Windows 10");
        }

    }
}
