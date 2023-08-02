use crate::{
    add_in::AddIn,
    ffi::{
        connection::Connection,
        types::{ParamValue, ReturnValue, Tm},
    },
};
use utf16_lit::utf16_null;

const PROPS: &[&[u16]] = &[
    &utf16_null!("Test"),
    &utf16_null!("PropI32"),
    &utf16_null!("PropF64"),
    &utf16_null!("PropBool"),
    &utf16_null!("PropDate"),
    &utf16_null!("PropStr"),
    &utf16_null!("PropBlob"),
];

const METHODS: &[&[u16]] = &[&utf16_null!("Method1"), &utf16_null!("ExtEvent")];

pub struct MyAddIn {
    test: i32,
    prop_i32: i32,
    prop_f64: f64,
    prop_bool: bool,
    prop_date: Tm,
    prop_str: String,
    prop_blob: Vec<u8>,
    connection: Option<&'static Connection>,
}

impl MyAddIn {
    pub fn new() -> MyAddIn {
        MyAddIn {
            test: 11111,
            prop_i32: 22222,
            prop_f64: 333.33,
            prop_bool: false,
            prop_date: Tm::default(),
            prop_str: String::from("00000"),
            prop_blob: Vec::new(),
            connection: None,
        }
    }
}

impl Drop for MyAddIn {
    fn drop(&mut self) {}
}

impl AddIn for MyAddIn {
    fn init(&mut self, _interface: &'static Connection) -> bool {
        self.connection = Some(_interface);
        true
    }

    fn done(&mut self) {}

    fn register_extension_as(&mut self) -> &'static [u16] {
        &utf16_null!("Class1")
    }

    fn get_n_props(&mut self) -> usize {
        PROPS.len()
    }

    fn find_prop(&mut self, name: &[u16]) -> Option<usize> {
        PROPS.iter().position(|&x| x == name)
    }

    fn get_prop_name(&mut self, num: usize, _alias: usize) -> Option<&'static [u16]> {
        PROPS.get(num).map(|&x| x)
    }

    fn get_prop_val(&mut self, num: usize, val: ReturnValue) -> bool {
        match num {
            0 => val.set_i32(self.test),
            1 => val.set_i32(self.prop_i32),
            2 => val.set_f64(self.prop_f64),
            3 => val.set_bool(self.prop_bool),
            4 => val.set_date(self.prop_date),
            5 => {
                let s: Vec<u16> = self.prop_str.encode_utf16().collect();
                val.set_str(s.as_slice());
            }
            6 => {
                val.set_blob(self.prop_blob.as_slice());
            }
            _ => return false,
        };
        true
    }

    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool {
        match num {
            0 => match val {
                ParamValue::I32(x) => {
                    self.test = *x;
                    true
                }
                _ => false,
            },
            1 => match val {
                ParamValue::I32(x) => {
                    self.prop_i32 = *x;
                    true
                }
                _ => false,
            },
            2 => match val {
                ParamValue::F64(x) => {
                    self.prop_f64 = *x;
                    true
                }
                _ => false,
            },
            3 => match val {
                ParamValue::Bool(x) => {
                    self.prop_bool = *x;
                    true
                }
                _ => false,
            },
            4 => match val {
                ParamValue::Date(x) => {
                    self.prop_date = *x;
                    true
                }
                _ => false,
            },
            5 => match val {
                ParamValue::Str(x) => {
                    self.prop_str = String::from_utf16(x).unwrap();
                    true
                }
                _ => false,
            },
            6 => match val {
                ParamValue::Blob(x) => {
                    self.prop_blob.clear();
                    self.prop_blob.extend_from_slice(x);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn is_prop_readable(&mut self, _num: usize) -> bool {
        true
    }

    fn is_prop_writable(&mut self, num: usize) -> bool {
        match num {
            0 => true,
            1 => true,
            2 => true,
            3 => true,
            4 => true,
            5 => true,
            6 => true,
            _ => false,
        }
    }

    fn get_n_methods(&mut self) -> usize {
        METHODS.len()
    }

    fn find_method(&mut self, name: &[u16]) -> Option<usize> {
        METHODS.iter().position(|&x| x == name)
    }

    fn get_method_name(&mut self, num: usize, _alias: usize) -> Option<&'static [u16]> {
        METHODS.get(num).map(|&x| x)
    }

    fn get_n_params(&mut self, num: usize) -> usize {
        match num {
            0 => 3,
            1 => 0,
            _ => 0,
        }
    }

    fn get_param_def_value(
        &mut self,
        _method_num: usize,
        _param_num: usize,
        _value: ReturnValue,
    ) -> bool {
        true
    }

    fn has_ret_val(&mut self, num: usize) -> bool {
        match num {
            0 => true,
            1 => true,
            _ => false,
        }
    }

    fn call_as_proc(&mut self, _num: usize, _params: &[ParamValue]) -> bool {
        false
    }

    fn call_as_func(&mut self, num: usize, params: &[ParamValue], ret_value: ReturnValue) -> bool {
        match num {
            0 => {
                let mut buf = Vec::<u16>::new();
                for p in params {
                    match p {
                        ParamValue::Str(x) => buf.extend_from_slice(x),
                        _ => return false,
                    }
                }
                ret_value.set_str(buf.as_slice());
                true
            }
            1 => {
                let Some(con) = self.connection else { return false };
                let res = con.external_event("asd", "asd", "asd");

                ret_value.set_bool(res);
                true
            }
            _ => false,
        }
    }

    fn set_locale(&mut self, _loc: &[u16]) {}

    fn set_user_interface_language_code(&mut self, _lang: &[u16]) {}
}
