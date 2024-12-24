use std::collections::HashMap;

use serde::{de, Serialize};
use windows_core::{BSTR, HSTRING, VARIANT};

use crate::{
    de::meta::struct_name_and_fields, result_enumerator::IWbemClassWrapper,
    ser::variant_ser::VariantStructSerializer, Variant, WMIConnection, WMIError, WMIResult,
};

impl WMIConnection {
    pub fn exec_method_native_wrapper(
        &self,
        method_class: impl AsRef<str>,
        object_path: impl AsRef<str>,
        method: impl AsRef<str>,
        in_params: &HashMap<String, Variant>,
    ) -> WMIResult<Option<IWbemClassWrapper>> {
        let method_class = BSTR::from(method_class.as_ref());
        let object_path = BSTR::from(object_path.as_ref());
        let method = BSTR::from(method.as_ref());

        // See https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemclassobject-getmethod
        // GetMethod can only be called on a class definition, so we retrieve that before retrieving a specific object
        let mut class_definition = None;
        unsafe {
            self.svc.GetObject(
                &method_class,
                Default::default(),
                &self.ctx.0,
                Some(&mut class_definition),
                None,
            )?;
        }
        let class_definition = class_definition.ok_or(WMIError::ResultEmpty)?;
        // The fields of the resulting IWbemClassObject will have the names and types of the WMI method's input parameters
        let mut input_signature = None;
        unsafe {
            class_definition.GetMethod(
                &method,
                Default::default(),
                &mut input_signature,
                std::ptr::null_mut(),
            )?;
        }
        let in_params = match input_signature {
            Some(input) => {
                let inst;
                unsafe {
                    inst = input.SpawnInstance(Default::default())?;
                };
                for (wszname, value) in in_params {
                    let wszname = HSTRING::from(wszname);
                    let value = TryInto::<VARIANT>::try_into(value.clone())?;

                    // See https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemclassobject-put
                    // Note that the example shows the variant being cleared (dropped) after the call to Put,
                    // so passing &value is acceptable here
                    unsafe {
                        inst.Put(&wszname, Default::default(), &value, 0)?;
                    }
                }
                Some(inst)
            }
            None => None,
        };

        let mut output = None;
        unsafe {
            self.svc.ExecMethod(
                &object_path,
                &method,
                Default::default(),
                &self.ctx.0,
                in_params.as_ref(),
                Some(&mut output),
                None,
            )?;
        }

        Ok(output.map(IWbemClassWrapper::new))
    }

    pub fn exec_class_method<MethodClass, In, Out>(
        &self,
        method: impl AsRef<str>,
        in_params: In,
    ) -> WMIResult<Out>
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
    ) -> WMIResult<Out>
    where
        MethodClass: de::DeserializeOwned,
        In: Serialize,
        Out: de::DeserializeOwned,
    {
        let (method_class, _) = struct_name_and_fields::<MethodClass>()?;
        let serializer = VariantStructSerializer::new();
        match in_params.serialize(serializer) {
            Ok(field_map) => {
                let output =
                    self.exec_method_native_wrapper(method_class, object_path, method, &field_map)?;

                match output {
                    Some(class_wrapper) => Ok(class_wrapper.into_desr()?),
                    None => Out::deserialize(Variant::Empty),
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

    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct CreateOutput {
        ReturnValue: u32,
        ProcessId: u32,
    }

    #[test]
    fn it_exec_methods() {
        // Create notepad instance
        let wmi_con = wmi_con();
        let in_params = CreateParams {
            CommandLine: "notepad.exe".to_string(),
        };
        let out = wmi_con
            .exec_class_method::<Win32_Process, CreateParams, CreateOutput>("Create", in_params)
            .unwrap();

        assert_eq!(out.ReturnValue, 0);

        let process = wmi_con
            .raw_query::<Win32_Process>(format!(
                "SELECT * FROM Win32_Process WHERE ProcessId = {}",
                out.ProcessId
            ))
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        wmi_con
            .exec_instance_method::<Win32_Process, (), ()>("Terminate", process.__Path, ())
            .unwrap();

        assert!(
            wmi_con
                .raw_query::<Win32_Process>(format!(
                    "SELECT * FROM Win32_Process WHERE ProcessId = {}",
                    out.ProcessId
                ))
                .unwrap()
                .len()
                == 0
        );
    }
}
