use crate::ffi::{
    connection::Connection,
    types::{ParamValue, ReturnValue},
};
use color_eyre::eyre::{Result};

pub struct ComponentPropDescription {
    pub name: &'static [u16],
    pub readable: bool,
    pub writable: bool,
}

pub struct ComponentFuncDescription {
    pub names: Vec<&'static [u16]>,
    pub params_count: usize,
    pub returns_val: bool,
    pub default_values: &'static [Option<ParamValue>],
}
impl ComponentFuncDescription {
    pub fn new<const PARAMS_COUNT: usize>(
        names: Vec<&'static [u16]>,
        returns_val: bool,
        default_values: &'static [Option<ParamValue>; PARAMS_COUNT],
    ) -> Self {
        Self {
            names,
            params_count: PARAMS_COUNT,
            returns_val,
            default_values,
        }
    }

    pub fn str_names(&self ) -> Vec<String> {
        self.names.iter().map(|name_utf16| {            
            let name_string = String::from_utf16_lossy(name_utf16).trim_matches(char::from(0)).to_string();
            name_string
        }).collect()        
    }
}

pub trait AddIn {
    fn init(&mut self, interface: &'static Connection) -> bool;
    fn add_in_name(&self) -> &'static [u16];
    fn list_parameters(&self) -> Vec<ComponentPropDescription> {
        Vec::new()
    }
    fn list_functions(&self) -> Vec<&ComponentFuncDescription> {
        Vec::new()
    }
    fn get_parameter(&self, _name: &str) -> Option<ParamValue> {
        None
    }
    fn set_parameter(&mut self, _name: &str, _value: &ParamValue) -> bool {
        false
    }
    fn call_function(&mut self, _name: &str, _params: &[ParamValue]) -> Result<Option<ParamValue>> {
        Ok(None)
    }
}

pub struct AddInContainer<T: AddIn> {
    add_in: T,
}

impl<T: AddIn> AddInContainer<T> {
    pub fn new(add_in: T) -> Self {
        Self { add_in }
    }
}

impl<T: AddIn> AddInWrapper for AddInContainer<T> {
    fn init(&mut self, interface: &'static Connection) -> bool {
        self.add_in.init(interface)
    }

    /// default 2000, don't use version 1000, because static objects are created
    fn get_info(&self) -> u16 {
        2000
    }

    fn done(&mut self) {}

    fn register_extension_as(&mut self) -> &'static [u16] {
        self.add_in.add_in_name()
    }

    fn get_n_props(&self) -> usize {
        self.add_in.list_parameters().len()
    }

    fn find_prop(&self, name: &[u16]) -> Option<usize> {
        self.add_in
            .list_parameters()
            .iter()
            .position(|x| x.name == name)
    }

    fn get_prop_name(&self, num: usize, alias: usize) -> Option<&'static [u16]> {
        self.add_in
            .list_parameters()
            .get(num)
            .map(|x| x.name)
    }

    fn get_prop_val(&self, num: usize, val: ReturnValue) -> bool {
        let param_desc_binding = self.add_in.list_parameters();
        let Some(param_desc) = param_desc_binding.get(num) else { 
            return false 
        };
        if !param_desc.readable {
            return false;
        };

        let name_string = String::from_utf16_lossy(param_desc.name);
        let name_utf8 = name_string.as_str().trim_matches(char::from(0));
        let Some(param_data) = self.add_in.get_parameter(name_utf8) else { 
            return false 
        };

        match param_data {
            ParamValue::I32(data_unwrapped) => val.set_i32(data_unwrapped),
            ParamValue::Bool(data_unwrapped) => val.set_bool(data_unwrapped),
            ParamValue::F64(data_unwrapped) => val.set_f64(data_unwrapped),
            ParamValue::Date(data_unwrapped) => val.set_date(data_unwrapped),
            ParamValue::Str(data_unwrapped) => val.set_str(&data_unwrapped),
            ParamValue::Blob(data_unwrapped) => val.set_blob(&data_unwrapped),
            ParamValue::Empty => val.set_empty(),
        }
        true
    }

    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool {
        let param_desc_binding = self.add_in.list_parameters();
        let Some(param_desc) = param_desc_binding.get(num) else { 
            return false 
        };

        if !param_desc.writable {
            return false;
        };

        let name_string = String::from_utf16_lossy(param_desc.name);
        let name_utf8 = name_string.as_str().trim_matches(char::from(0));
        self.add_in.set_parameter(name_utf8, val)
    }

    fn is_prop_readable(&self, num: usize) -> bool {
        let param_desc_binding = self.add_in.list_parameters();
        let Some(param_desc) = param_desc_binding.get(num) else { 
            return false 
        };
        param_desc.readable
    }

    fn is_prop_writable(&self, num: usize) -> bool {
        let param_desc_binding = self.add_in.list_parameters();
        let Some(param_desc) = param_desc_binding.get(num) else { 
            return false 
        };
        param_desc.writable
    }

    fn get_n_methods(&self) -> usize {
        self.add_in.list_functions().len()
    }

    fn find_method(&self, name: &[u16]) -> Option<usize> {
        self.add_in
            .list_functions()
            .iter()
            .position(|x| x.names.contains(&name))
    }

    fn get_method_name(&self, num: usize, alias: usize) -> Option<&'static [u16]> {
        self.add_in
            .list_functions()
            .get(num)
            .map(|x| x.names[0])
    }

    fn get_n_params(&self, num: usize) -> usize {
        let binding = self.add_in.list_functions();
        let Some(func_desc) = binding.get(num) else { 
            return 0 
        };
        func_desc.params_count
    }

    fn get_param_def_value(
        &self,
        method_num: usize,
        param_num: usize,
        value: ReturnValue,
    ) -> bool {
        let func_desc_binding = self.add_in.list_functions();
        let Some(func_desc) = func_desc_binding.get(method_num) else { 
            return false 
        };
        let Some(default_value_option) = func_desc.default_values.get(param_num) else { 
            return false 
        };
        let Some(default_value) = default_value_option else { 
            return false 
        };

        match default_value {
            ParamValue::Bool(data) => value.set_bool(*data),
            ParamValue::I32(data) => value.set_i32(*data),
            ParamValue::F64(data) => value.set_f64(*data),
            ParamValue::Date(data) => value.set_date(*data),
            ParamValue::Str(data) => value.set_str(data),
            ParamValue::Blob(data) => value.set_blob(data),
            ParamValue::Empty => value.set_empty(),
        }
        true
    }

    fn has_ret_val(&self, method_num: usize) -> bool {
        let func_desc_binding = self.add_in.list_functions();
        let Some(func_desc) = func_desc_binding.get(method_num) else { 
            return false 
        };
        func_desc.returns_val
    }

    fn call_as_proc(&mut self, method_num: usize, params: &[ParamValue]) -> bool {
        let func_desc_binding = self.add_in.list_functions();
        let Some(func_desc) = func_desc_binding.get(method_num) else { 
            return false 
        };

        let name_string = String::from_utf16_lossy(func_desc.names[0]);
        let name_utf8 = name_string.as_str().trim_matches(char::from(0));
        let call_result = match self.add_in.call_function(name_utf8, params) {
            Ok(r) => r,
            Err(err) => return false,
        };
        true
    }

    fn call_as_func(
        &mut self,
        method_num: usize,
        params: &[ParamValue],
        val: ReturnValue,
    ) -> bool {
        let func_desc_binding = self.add_in.list_functions();
        let Some(func_desc) = func_desc_binding.get(method_num) else { 
            return false 
        };
        if !func_desc.returns_val {
            return false;
        }

        let name_string = String::from_utf16_lossy(func_desc.names[0]);
        let name_utf8 = name_string.as_str().trim_matches(char::from(0));
        let call_result = match self.add_in.call_function(name_utf8, params) {
            Ok(r) => r,
            Err(err) => return false,
        };
        let Some(return_value) = call_result else {return false};
        match return_value {
            ParamValue::Bool(data) => val.set_bool(data),
            ParamValue::I32(data) => val.set_i32(data),
            ParamValue::F64(data) => val.set_f64(data),
            ParamValue::Date(data) => val.set_date(data),
            ParamValue::Str(data) => val.set_str(&data),
            ParamValue::Blob(data) => val.set_blob(&data),
            ParamValue::Empty => val.set_empty(),
        }
        true
    }
    fn set_locale(&mut self, _loc: &[u16]) {}
    fn set_user_interface_language_code(&mut self, _lang: &[u16]) {}
}

pub trait AddInWrapper {
    fn init(&mut self, interface: &'static Connection) -> bool;

    /// default 2000, don't use version 1000, because static objects are created
    fn get_info(&self) -> u16 {
        2000
    }
    fn done(&mut self);
    fn register_extension_as(&mut self) -> &'static [u16];
    fn get_n_props(&self) -> usize;
    fn find_prop(&self, name: &[u16]) -> Option<usize>;
    fn get_prop_name(&self, num: usize, alias: usize) -> Option<&'static [u16]>;
    fn get_prop_val(&self, num: usize, val: ReturnValue) -> bool;
    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool;
    fn is_prop_readable(&self, num: usize) -> bool;
    fn is_prop_writable(&self, num: usize) -> bool;
    fn get_n_methods(&self) -> usize;
    fn find_method(&self, name: &[u16]) -> Option<usize>;
    fn get_method_name(&self, num: usize, alias: usize) -> Option<&'static [u16]>;
    fn get_n_params(&self, num: usize) -> usize;
    fn get_param_def_value(
        &self,
        method_num: usize,
        param_num: usize,
        value: ReturnValue,
    ) -> bool;
    fn has_ret_val(&self, method_num: usize) -> bool;
    fn call_as_proc(&mut self, method_num: usize, params: &[ParamValue]) -> bool;
    fn call_as_func(
        &mut self,
        method_num: usize,
        params: &[ParamValue],
        val: ReturnValue,
    ) -> bool;
    fn set_locale(&mut self, loc: &[u16]);
    fn set_user_interface_language_code(&mut self, lang: &[u16]);
}
