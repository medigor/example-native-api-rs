use crate::ffi::{
    connection::Connection,
    types::{ParamValue, ReturnValue},
};

pub trait AddIn {
    fn init(&mut self, interface: &'static Connection) -> bool;

    /// default 2000, don't use version 1000, because static objects are created
    fn get_info(&mut self) -> u16 {
        2000
    }
    fn done(&mut self);
    fn register_extension_as(&mut self) -> &'static [u16];
    fn get_n_props(&mut self) -> usize;
    fn find_prop(&mut self, name: &[u16]) -> Option<usize>;
    fn get_prop_name(&mut self, num: usize, alias: usize) -> Option<&'static [u16]>;
    fn get_prop_val(&mut self, num: usize, val: ReturnValue) -> bool;
    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool;
    fn is_prop_readable(&mut self, num: usize) -> bool;
    fn is_prop_writable(&mut self, num: usize) -> bool;
    fn get_n_methods(&mut self) -> usize;
    fn find_method(&mut self, name: &[u16]) -> Option<usize>;
    fn get_method_name(&mut self, num: usize, alias: usize) -> Option<&'static [u16]>;
    fn get_n_params(&mut self, num: usize) -> usize;
    fn get_param_def_value(
        &mut self,
        method_num: usize,
        param_num: usize,
        value: ReturnValue,
    ) -> bool;
    fn has_ret_val(&mut self, method_num: usize) -> bool;
    fn call_as_proc(&mut self, method_num: usize, params: &[ParamValue]) -> bool;
    fn call_as_func(&mut self, method_num: usize, params: &[ParamValue], val: ReturnValue) -> bool;
    fn set_locale(&mut self, loc: &[u16]);
    fn set_user_interface_language_code(&mut self, lang: &[u16]);
}
