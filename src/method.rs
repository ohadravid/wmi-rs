use serde::{de, Serialize};
use windows::core::BSTR;

use crate::{
    de::meta::struct_name_and_fields, result_enumerator::IWbemClassWrapper,
    ser::variant_ser::VariantSerializer, Variant, WMIConnection, WMIError, WMIResult,
};

impl WMIConnection {
    /// Wrapper for WMI's [ExecMethod](https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execmethod) function.
    ///
    /// This function is used internally by [`WMIClass::exec_class_method`] and [`WMIClass::exec_instance_method`],
    /// which are a higher-level abstraction, dealing with Rust data types instead of raw Variants, that should be preferred to use.
    ///
    /// In the case of a class ("static") method, `object_path` should be name of the class.
    ///
    /// Returns `None` if the method has no out parameters and a `void` return type, and an [`IWbemClassWrapper`] containing the output otherwise.
    /// A method with a return type other than `void` will always have a generic property named `ReturnValue` in the output class wrapper with the return value of the WMI method call.
    ///
    /// ```edition2021
    /// # use std::collections::HashMap;
    /// # use wmi::{COMLibrary, Variant, WMIConnection, WMIResult};
    /// # fn main() -> WMIResult<()> {
    /// # let wmi_con = WMIConnection::new(COMLibrary::new()?)?;
    /// let in_params = wmi_con
    ///     .get_object("Win32_Process")?
    ///     .get_method("Create")?
    ///     .unwrap()
    ///     .spawn_instance()?;
    /// in_params.put_property("CommandLine", "explorer.exe".to_string())?;
    ///
    /// // Because Create has a return value and out parameters, the Option returned will never be None.
    /// // Note: The Create call can be unreliable, so consider using another means of starting processes.
    /// let out = wmi_con.exec_method("Win32_Process", "Create", Some(&in_params))?.unwrap();
    /// println!("The return code of the Create call is {:?}", out.get_property("ReturnValue")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn exec_method(
        &self,
        object_path: impl AsRef<str>,
        method: impl AsRef<str>,
        in_params: Option<&IWbemClassWrapper>,
    ) -> WMIResult<Option<IWbemClassWrapper>> {
        let object_path = BSTR::from(object_path.as_ref());

        // In the case of a method with no out parameters and a VOID return type, there will be no out-parameters object
        let method = BSTR::from(method.as_ref());

        let mut output = None;
        unsafe {
            self.svc.ExecMethod(
                &object_path,
                &method,
                Default::default(),
                &self.ctx.0,
                in_params.as_ref().map(|param| &param.inner),
                Some(&mut output),
                None,
            )?;
        }

        Ok(output.map(IWbemClassWrapper::new))
    }

    /// Specify a class using a type, for executing methods.
    /// See [`WMIClass::exec_class_method`] and [`WMIClass::exec_instance_method`] for an example.
    pub fn with_class<'a, Class>(&'a self) -> WMIResult<WMIClass<'a>>
    where
        Class: de::DeserializeOwned,
    {
        let (class, _) = struct_name_and_fields::<Class>()?;

        self.with_class_by_name(class)
    }

    /// Specify a class by name, for executing methods.
    /// See [`WMIClass::exec_class_method`] and [`WMIClass::exec_instance_method`] for an example.
    pub fn with_class_by_name<'a>(&'a self, class: &'a str) -> WMIResult<WMIClass<'a>> {
        Ok(WMIClass { wmi: self, class })
    }
}

#[derive(Clone, Debug)]
pub struct WMIClass<'a> {
    wmi: &'a WMIConnection,
    class: &'a str,
}

impl<'a> WMIClass<'a> {
    /// Executes a method of a WMI class not tied to any specific instance. Examples include
    /// [Create](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/create-method-in-class-win32-process) of `Win32_Process`
    /// and [AddPrinterConnection](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/addprinterconnection-method-in-class-win32-printer) of `Win32_Printer`.
    ///
    /// `In` and `Out` can custom structs supporting (de)serialization containing the input and output parameters of the function.
    ///
    /// Note: The `In` struct's name must deserialize to the invoked method's name,
    /// since the method signature also defines a type which is used for passing in the parameters of the call.
    ///
    /// A method with a return type other than `void` will always try to populate a generic property named `ReturnValue` in the output object with the return value of the WMI method call.
    /// If the method call has a `void` return type and no out parameters, the only acceptable type for `Out` is `()`.
    ///
    /// Arrays, Options, unknowns, and nested objects cannot be passed as input parameters due to limitations in how variants are constructed by `windows-rs`.
    ///
    /// This function uses [`WMIConnection::exec_method`] internally, with the name of the method class being the instance path, as is expected by WMI.
    ///
    /// ```edition2021
    /// # use serde::{Deserialize, Serialize};
    /// # use wmi::{COMLibrary, Variant, WMIConnection, WMIResult};
    /// #[derive(Serialize, Deserialize)]
    /// # #[allow(non_snake_case)]
    /// struct Create {
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
    /// let input = Create {
    ///     CommandLine: "explorer.exe".to_string()
    /// };
    /// let output: CreateOutput = wmi_con
    ///     .with_class::<Win32_Process>()?
    ///     .exec_class_method(input)?;
    ///
    /// println!("The return code of the Create call is {}", output.ReturnValue);
    /// println!("The ID of the created process is: {}", output.ProcessId);
    /// # Ok(())
    /// # }
    /// ```
    pub fn exec_class_method<In, Out>(&self, in_params: In) -> WMIResult<Out>
    where
        In: Serialize + de::DeserializeOwned,
        Out: de::DeserializeOwned,
    {
        self.exec_instance_method(self.class, in_params)
    }

    /// Executes a WMI method on a specific instance of a class. Examples include
    /// [GetSupportedSize](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/msft-Volume-getsupportedsizes) of `MSFT_Volume`
    /// and [Pause](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/pause-method-in-class-win32-printer) of `Win32_Printer`.
    ///
    /// `In` and `Out` can be `()` or any custom structs supporting (de)serialization containing the input and output parameters of the function.
    ///
    /// Note: The `In` struct's name must deserialize to the invoked method's name,
    /// since the method signature also defines a type which is used for passing in the parameters of the call.
    ///
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
    /// #[derive(Serialize, Deserialize)]
    /// # #[allow(non_snake_case)]
    /// struct Pause;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// # #[allow(non_snake_case)]
    /// struct Resume;
    ///
    /// # fn main() -> WMIResult<()> {
    /// # let wmi_con = WMIConnection::new(COMLibrary::new()?)?;
    /// let printers: Vec<Win32_Printer> = wmi_con.query()?;
    ///
    /// let wmi_printer_cls = wmi_con.with_class::<Win32_Printer>()?;
    ///
    /// for printer in printers {
    ///     let output: PrinterOutput = wmi_printer_cls.exec_instance_method(&printer.__Path, Pause)?;
    ///     println!("Pausing the printer returned {}", output.ReturnValue);
    ///
    ///     let output: PrinterOutput = wmi_printer_cls.exec_instance_method(&printer.__Path, Resume)?;
    ///     println!("Resuming the printer returned {}", output.ReturnValue);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn exec_instance_method<In, Out>(
        &self,
        object_path: impl AsRef<str>,
        in_params: In,
    ) -> WMIResult<Out>
    where
        In: Serialize + de::DeserializeOwned,
        Out: de::DeserializeOwned,
    {
        let (method, _) = struct_name_and_fields::<In>()?;
        let serializer = VariantSerializer {
            wmi: self.wmi,
            class: Some(self.class),
        };

        let instance = match in_params.serialize(serializer) {
            Ok(Variant::Object(instance)) => Some(instance),
            Ok(Variant::Empty) => None,
            Ok(other) => {
                return Err(WMIError::ConvertVariantError(format!(
                    "Unexpected serializer output: {:?}",
                    other
                )))
            }
            Err(e) => return Err(WMIError::ConvertVariantError(e.to_string())),
        };

        let output = self
            .wmi
            .exec_method(object_path, method, instance.as_ref())?;

        match output {
            Some(class_wrapper) => Ok(class_wrapper.into_desr()?),
            None => Out::deserialize(Variant::Empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::wmi_con;
    use crate::Variant;
    use serde::{Deserialize, Serialize};
    use std::thread::sleep;
    use std::time::Duration;

    #[derive(Deserialize)]
    struct Win32_Process {
        __Path: String,
    }

    #[derive(Deserialize, Serialize)]
    struct Create {
        CommandLine: String,
    }

    #[derive(Deserialize, Serialize)]
    struct Terminate {}

    #[derive(Deserialize)]
    struct CreateOutput {
        ReturnValue: u32,
        ProcessId: u32,
    }

    #[test]
    fn it_exec_methods_native() {
        let wmi_con = wmi_con();

        let in_params = wmi_con
            .get_object("Win32_Process")
            .unwrap()
            .get_method("Create")
            .unwrap()
            .unwrap()
            .spawn_instance()
            .unwrap();

        in_params
            .put_property("CommandLine", "explorer.exe".to_string())
            .unwrap();

        let out = wmi_con
            .exec_method("Win32_Process", "Create", Some(&in_params))
            .unwrap();

        let return_value = out.unwrap().get_property("ReturnValue").unwrap();

        assert!(matches!(return_value, Variant::UI4(0)));
    }

    #[test]
    fn it_exec_methods() {
        let wmi_con = wmi_con();
        let in_params = Create {
            CommandLine: "explorer.exe".to_string(),
        };
        let out: CreateOutput = wmi_con
            .with_class::<Win32_Process>()
            .unwrap()
            .exec_class_method(in_params)
            .unwrap();

        assert_eq!(out.ReturnValue, 0);

        let query = format!(
            "SELECT * FROM Win32_Process WHERE ProcessId = {}",
            out.ProcessId
        );

        let process = &wmi_con.raw_query::<Win32_Process>(&query).unwrap()[0];

        let _: () = wmi_con
            .with_class::<Win32_Process>()
            .unwrap()
            .exec_instance_method(&process.__Path, Terminate {})
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

    #[test]
    fn it_exec_with_u8_arrays() {
        let wmi_con = wmi_con();

        #[derive(Deserialize, Serialize)]
        struct GetBinaryValue {
            sSubKeyName: String,
            sValueName: String,
        }

        #[derive(Deserialize)]
        struct GetBinaryValueOut {
            uValue: Vec<u8>,
        }

        let get_binary_value_params = GetBinaryValue {
            sSubKeyName: r#"SYSTEM\CurrentControlSet\Control\Windows"#.to_string(),
            sValueName: "FullProcessInformationSID".to_string(),
        };

        let value: GetBinaryValueOut = wmi_con
            .with_class_by_name("StdRegProv")
            .unwrap()
            .exec_class_method(get_binary_value_params)
            .unwrap();

        assert!(value.uValue.len() > 0, "Expected to get a non-empty value");
    }
}
