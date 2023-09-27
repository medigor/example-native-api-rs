use crate::ffi::{Addin, Connection, ParamValue, Variant};
use utf16_lit::utf16_null;

pub struct Addin2 {
    test1: i32,
}

impl Addin2 {
    pub fn new() -> Addin2 {
        Addin2 { test1: 12345 }
    }
}

impl Drop for Addin2 {
    fn drop(&mut self) {}
}

impl Addin for Addin2 {
    fn init(&mut self, _interface: &'static Connection) -> bool {
        true
    }

    fn done(&mut self) {}

    fn register_extension_as(&mut self) -> &'static [u16] {
        &utf16_null!("Class2")
    }

    fn get_n_props(&mut self) -> usize {
        1
    }

    fn find_prop(&mut self, name: &[u16]) -> Option<usize> {
        const TEST1: &[u16] = &utf16_null!("Test1");
        match name {
            TEST1 => Some(0),
            _ => None,
        }
    }

    fn get_prop_name(&mut self, num: usize, _alias: usize) -> Option<&'static [u16]> {
        if num == 0 {
            Some(&utf16_null!("Test1"))
        } else {
            None
        }
    }

    fn get_prop_val(&mut self, num: usize, val: &mut Variant) -> bool {
        match num {
            0 => {
                val.set_i32(self.test1);
                true
            }
            _ => false,
        }
    }

    fn set_prop_val(&mut self, _num: usize, _val: &ParamValue) -> bool {
        false
    }

    fn is_prop_readable(&mut self, _num: usize) -> bool {
        true
    }

    fn is_prop_writable(&mut self, _num: usize) -> bool {
        false
    }

    fn get_n_methods(&mut self) -> usize {
        0
    }

    fn find_method(&mut self, _name: &[u16]) -> Option<usize> {
        None
    }

    fn get_method_name(&mut self, _num: usize, _alias: usize) -> Option<&'static [u16]> {
        None
    }

    fn get_n_params(&mut self, _num: usize) -> usize {
        0
    }

    fn get_param_def_value(
        &mut self,
        _method_num: usize,
        _param_num: usize,
        _value: Variant,
    ) -> bool {
        true
    }

    fn has_ret_val(&mut self, _method_num: usize) -> bool {
        false
    }

    fn call_as_proc(&mut self, _method_num: usize, _params: &mut [Variant]) -> bool {
        false
    }

    fn call_as_func(
        &mut self,
        _method_num: usize,
        _params: &mut [Variant],
        _val: &mut Variant,
    ) -> bool {
        false
    }

    fn set_locale(&mut self, _loc: &[u16]) {}

    fn set_user_interface_language_code(&mut self, _lang: &[u16]) {}
}
