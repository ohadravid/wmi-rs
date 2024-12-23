use std::collections::HashMap;

use serde::{de, Serialize};
use windows_core::{BSTR, HSTRING, VARIANT};

use crate::{
    de::meta::struct_name_and_fields, result_enumerator::IWbemClassWrapper,
    ser::variant_ser::VariantStructSerializer, WMIConnection, WMIError, WMIResult,
};

impl WMIConnection {
    fn exec_method_native_wrapper(
        &self,
        method_class: impl AsRef<str>,
        object_path: impl AsRef<str>,
        method: impl AsRef<str>,
        in_params: HashMap<String, VARIANT>,
    ) -> WMIResult<Option<IWbemClassWrapper>> {
        let method_class = BSTR::from(method_class.as_ref());
        let object_path = BSTR::from(object_path.as_ref());
        let method = BSTR::from(method.as_ref());

        unsafe {
            let mut class_object = None;
            self.svc.GetObject(
                &method_class,
                Default::default(),
                &self.ctx.0,
                Some(&mut class_object),
                None,
            )?;

            match class_object {
                Some(class) => {
                    let mut input_signature = None;
                    class.GetMethod(
                        &method,
                        Default::default(),
                        &mut input_signature,
                        std::ptr::null_mut(),
                    )?;
                    let object = match input_signature {
                        Some(input) => {
                            let inst = input.SpawnInstance(0)?;
                            for (wszname, value) in in_params {
                                inst.Put(&HSTRING::from(wszname), Default::default(), &value, 0)?;
                            }
                            Some(inst)
                        }
                        None => None,
                    };

                    let mut output = None;
                    self.svc.ExecMethod(
                        &object_path,
                        &method,
                        Default::default(),
                        &self.ctx.0,
                        object.as_ref(),
                        Some(&mut output),
                        None,
                    )?;

                    match output {
                        Some(wbem_class_obj) => Ok(Some(IWbemClassWrapper::new(wbem_class_obj))),
                        None => Ok(None),
                    }
                }
                None => Err(WMIError::ResultEmpty),
            }
        }
    }

    pub fn exec_class_method<MethodClass, In, Out>(
        &self,
        method: impl AsRef<str>,
        in_params: In,
    ) -> WMIResult<Option<Out>>
    where
        MethodClass: de::DeserializeOwned,
        In: Serialize,
        Out: de::DeserializeOwned,
    {
        let (method_class, _) = struct_name_and_fields::<MethodClass>()?;
        self.exec_instance_method::<MethodClass, In, Out>(method, method_class, in_params)
    }

    pub fn exec_instance_method<MethodClass, In, Out>(
        &self,
        method: impl AsRef<str>,
        object_path: impl AsRef<str>,
        in_params: In,
    ) -> WMIResult<Option<Out>>
    where
        MethodClass: de::DeserializeOwned,
        In: Serialize,
        Out: de::DeserializeOwned,
    {
        let (method_class, _) = struct_name_and_fields::<MethodClass>()?;
        let serializer = VariantStructSerializer::new();
        match in_params.serialize(serializer) {
            Ok(field_map) => {
                let field_map: HashMap<String, VARIANT> = field_map
                    .into_iter()
                    .filter_map(|(k, v)| match TryInto::<VARIANT>::try_into(v).ok() {
                        Some(variant) => Some((k, variant)),
                        None => None,
                    })
                    .collect();
                let output =
                    self.exec_method_native_wrapper(method_class, object_path, method, field_map);

                match output {
                    Ok(wbem_class_obj) => match wbem_class_obj {
                        Some(wbem_class_obj) => Ok(Some(wbem_class_obj.into_desr()?)),
                        None => Ok(None),
                    },
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(WMIError::ConvertVariantError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::wmi_con;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    struct Win32_Process {
        __Path: String,
    }

    #[derive(Serialize)]
    struct CreateParams {
        CommandLine: String,
    }

    #[derive(Serialize)]
    struct TerminateParams {}

    #[derive(Deserialize)]
    struct TerminateOutput {}

    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct CreateOutput {
        ProcessId: u32,
    }

    #[test]
    fn it_exec_class_method() {
        let wmi_con = wmi_con();
        let in_params = CreateParams {
            CommandLine: "notepad.exe".to_string(),
        };

        let out = wmi_con
            .exec_class_method::<Win32_Process, CreateParams, CreateOutput>("Create", in_params)
            .unwrap()
            .unwrap();

        assert!(out.ProcessId != 0);
    }

    #[test]
    fn it_exec_instance_method() {
        // Create notepad instance
        let wmi_con = wmi_con();
        let in_params = CreateParams {
            CommandLine: "notepad.exe".to_string(),
        };
        let out = wmi_con
            .exec_class_method::<Win32_Process, CreateParams, CreateOutput>("Create", in_params)
            .unwrap()
            .unwrap();

        let process = wmi_con
            .raw_query::<Win32_Process>(format!(
                "SELECT * FROM Win32_Process WHERE ProcessId = {}",
                out.ProcessId
            ))
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        let _ = wmi_con
            .exec_instance_method::<Win32_Process, TerminateParams, TerminateOutput>(
                "Terminate",
                process.__Path,
                TerminateParams {},
            )
            .unwrap();
    }
}
