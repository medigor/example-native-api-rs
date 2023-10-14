use crate::ffi::{self, ParamValue, Variant};

#[allow(dead_code)]
pub enum Methods<T> {
    Method0(fn(&mut T, &mut Variant) -> bool),
    Method1(fn(&mut T, &mut Variant, &mut Variant) -> bool),
    Method2(fn(&mut T, &mut Variant, &mut Variant, &mut Variant) -> bool),
    Method3(fn(&mut T, &mut Variant, &mut Variant, &mut Variant, &mut Variant) -> bool),
    Method4(
        fn(&mut T, &mut Variant, &mut Variant, &mut Variant, &mut Variant, &mut Variant) -> bool,
    ),
    Method5(
        fn(
            &mut T,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
        ) -> bool,
    ),
    Method6(
        fn(
            &mut T,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
        ) -> bool,
    ),
    Method7(
        fn(
            &mut T,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
            &mut Variant,
        ) -> bool,
    ),
}

impl<T> Methods<T> {
    fn params(&self) -> usize {
        match self {
            Methods::Method0(_) => 0,
            Methods::Method1(_) => 1,
            Methods::Method2(_) => 2,
            Methods::Method3(_) => 3,
            Methods::Method4(_) => 4,
            Methods::Method5(_) => 5,
            Methods::Method6(_) => 6,
            Methods::Method7(_) => 7,
        }
    }

    #[allow(unused_variables)]
    fn call(&self, addin: &mut T, params: &mut [Variant], val: &mut Variant) -> bool {
        match self {
            Methods::Method0(f) => f(addin, val),
            Methods::Method1(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, val)
            }
            Methods::Method2(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p2, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, p2, val)
            }
            Methods::Method3(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p2, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p3, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, p2, p3, val)
            }
            Methods::Method4(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p2, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p3, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p4, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, p2, p3, p4, val)
            }

            Methods::Method5(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p2, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p3, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p4, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p5, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, p2, p3, p4, p5, val)
            }

            Methods::Method6(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p2, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p3, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p4, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p5, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p6, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, p2, p3, p4, p5, p6, val)
            }

            Methods::Method7(f) => {
                let Some((p1, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p2, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p3, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p4, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p5, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p6, params)) = params.split_first_mut() else {
                    return false;
                };
                let Some((p7, params)) = params.split_first_mut() else {
                    return false;
                };
                f(addin, p1, p2, p3, p4, p5, p6, p7, val)
            }
        }
    }
}

pub struct MethodInfo<T> {
    pub name: &'static [u16],
    pub method: Methods<T>,
}

pub struct PropInfo<T> {
    pub name: &'static [u16],
    pub getter: Option<fn(&mut T, &mut Variant) -> bool>,
    pub setter: Option<fn(&mut T, &ParamValue) -> bool>,
}

pub trait Addin {
    fn name() -> &'static [u16];

    fn get_info() -> u16 {
        2000
    }

    fn methods() -> &'static [MethodInfo<Self>]
    where
        Self: Sized,
    {
        &[]
    }

    fn properties() -> &'static [PropInfo<Self>]
    where
        Self: Sized,
    {
        &[]
    }
}

#[allow(unused_variables)]
impl<T: Addin + 'static> ffi::Addin for T {
    fn register_extension_as(&mut self) -> &'static [u16] {
        T::name()
    }

    fn get_info(&mut self) -> u16 {
        T::get_info()
    }

    fn get_n_props(&mut self) -> usize {
        T::properties().len()
    }

    fn find_prop(&mut self, name: &[u16]) -> Option<usize> {
        T::properties().iter().position(|x| x.name == name)
    }

    fn get_prop_name(&mut self, num: usize, alias: usize) -> Option<&'static [u16]> {
        T::properties().get(num).map(|x| &x.name).copied()
    }

    fn get_prop_val(&mut self, num: usize, val: &mut Variant) -> bool {
        let Some(property) = T::properties().get(num) else {
            return false;
        };
        let Some(getter) = property.getter else {
            return false;
        };
        getter(self, val)
    }

    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool {
        let Some(property) = T::properties().get(num) else {
            return false;
        };
        let Some(setter) = property.setter else {
            return false;
        };
        setter(self, val)
    }

    fn is_prop_readable(&mut self, num: usize) -> bool {
        T::properties()[num].getter.is_some()
    }

    fn is_prop_writable(&mut self, num: usize) -> bool {
        T::properties()[num].setter.is_some()
    }

    fn get_n_methods(&mut self) -> usize {
        T::methods().len()
    }

    fn find_method(&mut self, name: &[u16]) -> Option<usize> {
        T::methods().iter().position(|x| x.name == name)
    }

    fn get_method_name(&mut self, num: usize, alias: usize) -> Option<&'static [u16]> {
        T::methods().get(num).map(|x| &x.name).copied()
    }

    fn get_n_params(&mut self, num: usize) -> usize {
        let Some(info) = T::methods().get(num) else {
            return 0;
        };
        info.method.params()
    }

    fn get_param_def_value(&mut self, method_num: usize, param_num: usize, value: Variant) -> bool {
        true
    }

    fn has_ret_val(&mut self, method_num: usize) -> bool {
        true
    }

    fn call_as_proc(&mut self, method_num: usize, params: &mut [Variant]) -> bool {
        false
    }

    fn call_as_func(
        &mut self,
        method_num: usize,
        params: &mut [Variant],
        val: &mut Variant,
    ) -> bool {
        let Some(info) = T::methods().get(method_num) else {
            return false;
        };

        info.method.call(self, params, val)
    }
}
