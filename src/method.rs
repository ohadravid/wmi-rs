use std::collections::HashMap;

use serde::{de, Serialize};
use windows_core::{BSTR, HSTRING, VARIANT};

use crate::{
    de::meta::struct_name_and_fields, result_enumerator::IWbemClassWrapper,
    ser::variant_ser::VariantStructSerializer, Variant, WMIConnection, WMIError, WMIResult,
};

impl WMIConnection {
    /// Wrapper for WMI's [ExecMethod](https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execmethod) function.
    ///
    /// This function is used internally by [`WMIConnection::exec_class_method`] and [`WMIConnection::exec_instance_method`],
    /// which are a higher-level abstraction, dealing with Rust data types instead of raw Variants, that should be preferred to use.
    ///
    /// In the case of a class ("static") method, `object_path` should be the same as `method_class`.
    ///
    /// Returns `None` if the method has no out parameters and a `void` return type, and an [`IWbemClassWrapper`] containing the output otherwise.
    /// A method with a return type other than `void` will always have a generic property named `ReturnValue` in the output class wrapper with the return value of the WMI method call.
    ///
    /// ```edition2021
    /// # use wmi::{COMLibrary, Variant, WMIConnection, WMIResult};
    /// # fn main() -> WMIResult<()> {
    /// # let wmi_con = WMIConnection::new(COMLibrary::new()?)?;
    /// let in_params = [
    ///     ("CommandLine".to_string(), Variant::from("explorer.exe".to_string()))
    /// ].into_iter().collect();
    ///
    /// // Because Create has a return value and out parameters, the Option returned will never be None.
    /// // Note: The Create call can be unreliable, so consider using another means of starting processes.
    /// let out = wmi_con.exec_method_native_wrapper("Win32_Process", "Win32_Process", "Create", in_params)?.unwrap();
    /// println!("The return code of the Create call is {:?}", out.get_property("ReturnValue")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn exec_method_native_wrapper(
        &self,
        method_class: impl AsRef<str>,
        object_path: impl AsRef<str>,
        method: impl AsRef<str>,
        in_params: HashMap<String, Variant>,
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
        // Retrieve the input signature of the WMI method.
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

        // The method may have no input parameters, such as in this case: https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/reboot-method-in-class-win32-operatingsystem
        let in_params = match input_signature {
            Some(input) => {
                let inst;
                unsafe {
                    inst = input.SpawnInstance(Default::default())?;
                };
                // Set every field of the input object to the corresponding input parameter passed to this function
                for (wszname, value) in in_params {
                    let wszname = HSTRING::from(wszname);
                    let value = TryInto::<VARIANT>::try_into(value)?;

                    // See https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemclassobject-put
                    // Note that the example shows the variant is expected to be cleared (dropped) after the call to Put,
                    // so passing &value is acceptable here
                    unsafe {
                        inst.Put(&wszname, Default::default(), &value, 0)?;
                    }
                }
                Some(inst)
            }
            None => None,
        };

        // In the case of a method with no out parameters and a VOID return type, there will be no out-parameters object
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

    /// Executes a method of a WMI class not tied to any specific instance. Examples include
    /// [Create](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/create-method-in-class-win32-process) of `Win32_Process`
    /// and [AddPrinterConnection](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/addprinterconnection-method-in-class-win32-printer) of `Win32_Printer`.
    ///
    /// `MethodClass` should have the name of the class on which the method is being invoked.
    /// `In` and `Out` can be `()` or any custom structs supporting (de)serialization containing the input and output parameters of the function.
    ///
    /// A method with a return type other than `void` will always try to populate a generic property named `ReturnValue` in the output object with the return value of the WMI method call.
    /// If the method call has a `void` return type and no out parameters, the only acceptable type for `Out` is `()`.
    ///
    /// Arrays, Options, unknowns, and nested objects cannot be passed as input parameters due to limitations in how variants are constructed by `windows-rs`.
    ///
    /// This function uses [`WMIConnection::exec_instance_method`] internally, with the name of the method class being the instance path, as is expected by WMI.
    ///
    /// ```edition2021
    /// # use serde::{Deserialize, Serialize};
    /// # use wmi::{COMLibrary, Variant, WMIConnection, WMIResult};
    /// #[derive(Serialize)]
    /// # #[allow(non_snake_case)]
    /// struct CreateInput {
    ///     CommandLine: String
    /// }
    ///
    /// #[derive(Deserialize)]
    /// # #[allow(non_snake_case)]
    /// struct CreateOutput {
    ///     ReturnValue: u32,
    ///     ProcessId: u32
    /// }
    ///
    /// #[derive(Deserialize)]
    /// # #[allow(non_camel_case_types)]
    /// struct Win32_Process;
    ///
    /// # fn main() -> WMIResult<()> {
    /// # let wmi_con = WMIConnection::new(COMLibrary::new()?)?;
    /// // Note: The Create call can be unreliable, so consider using another means of starting processes.
    /// let input = CreateInput {
    ///     CommandLine: "explorer.exe".to_string()
    /// };
    /// let output: CreateOutput = wmi_con.exec_class_method::<Win32_Process, _, _>("Create", input)?;
    ///
    /// println!("The return code of the Create call is {}", output.ReturnValue);
    /// println!("The ID of the created process is: {}", output.ProcessId);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Executes a WMI method on a specific instance of a class. Examples include
    /// [GetSupportedSize](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/msft-Volume-getsupportedsizes) of `MSFT_Volume`
    /// and [Pause](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/pause-method-in-class-win32-printer) of `Win32_Printer`.
    ///
    /// `MethodClass` should have the name of the class on which the method is being invoked.
    /// `In` and `Out` can be `()` or any custom structs supporting (de)serialization containing the input and output parameters of the function.
    /// `object_path` is the `__Path` variable of the class instance on which the method is being called, which can be obtained from a WMI query.
    ///
    /// A method with a return type other than `void` will always try to populate a generic property named `ReturnValue` in the output object with the return value of the WMI method call.
    /// If the method call has a `void` return type and no out parameters, the only acceptable type for `Out` is `()`.
    ///
    /// Arrays, Options, unknowns, and nested objects cannot be passed as input parameters due to limitations in how variants are constructed by `windows-rs`.
    ///
    /// ```edition2021
    /// # use serde::{Deserialize, Serialize};
    /// # use wmi::{COMLibrary, FilterValue, Variant, WMIConnection, WMIResult};
    /// #[derive(Deserialize)]
    /// # #[allow(non_snake_case)]
    /// struct PrinterOutput {
    ///     ReturnValue: u32
    /// }
    ///
    /// #[derive(Deserialize)]
    /// # #[allow(non_camel_case_types, non_snake_case)]
    /// struct Win32_Printer {
    ///     __Path: String
    /// }
    ///
    /// # fn main() -> WMIResult<()> {
    /// # let wmi_con = WMIConnection::new(COMLibrary::new()?)?;
    /// let printers: Vec<Win32_Printer> = wmi_con.query()?;
    ///
    /// for printer in printers {
    ///     let output: PrinterOutput = wmi_con.exec_instance_method::<Win32_Printer, _, _>("Pause", &printer.__Path, ())?;
    ///     println!("Pausing the printer returned {}", output.ReturnValue);
    ///
    ///     let output: PrinterOutput = wmi_con.exec_instance_method::<Win32_Printer, _, _>("Resume", &printer.__Path, ())?;
    ///     println!("Resuming the printer returned {}", output.ReturnValue);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
                    self.exec_method_native_wrapper(method_class, object_path, method, field_map)?;

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
    use std::thread::sleep;
    use std::time::Duration;

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
        let wmi_con = wmi_con();
        let in_params = CreateParams {
            CommandLine: "explorer.exe".to_string(),
        };
        let out = wmi_con
            .exec_class_method::<Win32_Process, CreateParams, CreateOutput>("Create", in_params)
            .unwrap();

        assert_eq!(out.ReturnValue, 0);

        let query = format!(
            "SELECT * FROM Win32_Process WHERE ProcessId = {}",
            out.ProcessId
        );

        let process = &wmi_con.raw_query::<Win32_Process>(&query).unwrap()[0];

        wmi_con
            .exec_instance_method::<Win32_Process, (), ()>("Terminate", &process.__Path, ())
            .unwrap();

        // It can take a moment for the process to terminate, so we retry the query a few times.
        for _ in 0..10 {
            if wmi_con.raw_query::<Win32_Process>(&query).unwrap().len() == 0 {
                break;
            }
            sleep(Duration::from_millis(100));
        }

        assert!(wmi_con.raw_query::<Win32_Process>(&query).unwrap().len() == 0);
    }
}
