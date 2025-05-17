use addin1c::{name, CStr1C, ParamValue, RawAddin, Tm, Variant};

const PROPS: &[&CStr1C] = &[
    name!("Test"),
    name!("PropI32"),
    name!("PropF64"),
    name!("PropBool"),
    name!("PropDate"),
    name!("PropStr"),
    name!("PropBlob"),
];

const METHODS: &[&CStr1C] = &[name!("Method1"), name!("Method2")];

pub struct Addin1 {
    test: i32,
    prop_i32: i32,
    prop_f64: f64,
    prop_bool: bool,
    prop_date: Tm,
    prop_str: String,
    prop_blob: Vec<u8>,
}

impl Addin1 {
    pub fn new() -> Addin1 {
        Addin1 {
            test: 11111,
            prop_i32: 22222,
            prop_f64: 333.33,
            prop_bool: false,
            prop_date: Tm::default(),
            prop_str: String::from("00000"),
            prop_blob: Vec::new(),
        }
    }
}

impl Drop for Addin1 {
    fn drop(&mut self) {}
}

impl RawAddin for Addin1 {
    fn register_extension_as(&mut self) -> &'static CStr1C {
        name!("Class1")
    }

    fn get_n_props(&mut self) -> usize {
        PROPS.len()
    }

    fn find_prop(&mut self, name: &CStr1C) -> Option<usize> {
        PROPS.iter().position(|&x| x == name)
    }

    fn get_prop_name(&mut self, num: usize, _alias: usize) -> Option<&'static CStr1C> {
        PROPS.get(num).copied()
    }

    fn get_prop_val(&mut self, num: usize, val: &mut Variant) -> bool {
        match num {
            0 => val.set_i32(self.test),
            1 => val.set_i32(self.prop_i32),
            2 => val.set_f64(self.prop_f64),
            3 => val.set_bool(self.prop_bool),
            4 => val.set_date(self.prop_date),
            5 => {
                let s: Vec<u16> = self.prop_str.encode_utf16().collect();
                return val.set_str1c(s.as_slice()).is_ok();
            }
            6 => {
                return val.set_blob(self.prop_blob.as_slice()).is_ok();
            }
            _ => return false,
        };
        true
    }

    fn set_prop_val(&mut self, num: usize, val: &Variant) -> bool {
        match num {
            0 => match val.get() {
                ParamValue::I32(x) => {
                    self.test = x;
                    true
                }
                _ => false,
            },
            1 => match val.get() {
                ParamValue::I32(x) => {
                    self.prop_i32 = x;
                    true
                }
                _ => false,
            },
            2 => match val.get() {
                ParamValue::F64(x) => {
                    self.prop_f64 = x;
                    true
                }
                _ => false,
            },
            3 => match val.get() {
                ParamValue::Bool(x) => {
                    self.prop_bool = x;
                    true
                }
                _ => false,
            },
            4 => match val.get() {
                ParamValue::Date(x) => {
                    self.prop_date = x;
                    true
                }
                _ => false,
            },
            5 => match val.get() {
                ParamValue::Str(x) => {
                    self.prop_str = String::from_utf16(x).unwrap();
                    true
                }
                _ => false,
            },
            6 => match val.get() {
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
        matches!(num, 0..=6)
    }

    fn get_n_methods(&mut self) -> usize {
        METHODS.len()
    }

    fn find_method(&mut self, name: &CStr1C) -> Option<usize> {
        METHODS.iter().position(|&x| x == name)
    }

    fn get_method_name(&mut self, num: usize, _alias: usize) -> Option<&'static CStr1C> {
        METHODS.get(num).copied()
    }

    fn get_n_params(&mut self, num: usize) -> usize {
        match num {
            0 => 3,
            1 => 2,
            _ => 0,
        }
    }

    fn get_param_def_value(
        &mut self,
        _method_num: usize,
        _param_num: usize,
        _value: Variant,
    ) -> bool {
        true
    }

    fn has_ret_val(&mut self, num: usize) -> bool {
        matches!(num, 0 | 1)
    }

    fn call_as_proc(&mut self, _num: usize, _params: &mut [Variant]) -> bool {
        false
    }

    fn call_as_func(
        &mut self,
        num: usize,
        params: &mut [Variant],
        ret_value: &mut Variant,
    ) -> bool {
        match num {
            0 => {
                let mut buf = Vec::<u16>::new();
                for param in params {
                    match param.get() {
                        ParamValue::Str(x) => buf.extend_from_slice(x),
                        _ => return false,
                    }
                }
                ret_value.set_str1c(buf.as_slice()).is_ok()
            }
            1 => {
                for (i, param) in params.iter_mut().enumerate() {
                    match param.get() {
                        ParamValue::Empty => {
                            if i == 0 {
                                let s = "Return value".encode_utf16().collect::<Vec<u16>>();
                                if param.set_str1c(s.as_slice()).is_err() {
                                    return false;
                                }
                            } else {
                                param.set_i32(1)
                            }
                        }
                        _ => return false,
                    }
                }
                ret_value.set_bool(true);
                true
            }
            _ => false,
        }
    }
}
