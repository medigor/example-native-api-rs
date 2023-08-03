use std::{
    ffi::c_long,
    ptr::{self},
    slice::from_raw_parts,
};

use crate::add_in::AddInWrapper;

use super::{
    get_str,
    types::{ParamValue, ReturnValue, TVariant},
    This,
};

#[repr(C)]
pub struct LanguageExtenderBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    register_extension_as:
        unsafe extern "system" fn(&mut This<1, T>, *mut *mut u16) -> bool,
    get_n_props: unsafe extern "system" fn(&mut This<1, T>) -> c_long,
    find_prop: unsafe extern "system" fn(&mut This<1, T>, *const u16) -> c_long,
    get_prop_name: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        c_long,
    ) -> *const u16,
    get_prop_val: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        &mut TVariant,
    ) -> bool,
    set_prop_val:
        unsafe extern "system" fn(&mut This<1, T>, c_long, &TVariant) -> bool,
    is_prop_readable:
        unsafe extern "system" fn(&mut This<1, T>, c_long) -> bool,
    is_prop_writable:
        unsafe extern "system" fn(&mut This<1, T>, c_long) -> bool,
    get_n_methods: unsafe extern "system" fn(&mut This<1, T>) -> c_long,
    find_method:
        unsafe extern "system" fn(&mut This<1, T>, *const u16) -> c_long,
    get_method_name: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        c_long,
    ) -> *const u16,
    get_n_params: unsafe extern "system" fn(&mut This<1, T>, c_long) -> c_long,
    get_param_def_value: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        c_long,
        &mut TVariant,
    ) -> bool,
    has_ret_val: unsafe extern "system" fn(&mut This<1, T>, c_long) -> bool,
    call_as_proc: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        *const TVariant,
        c_long,
    ) -> bool,
    call_as_func: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        &mut TVariant,
        *const TVariant,
        c_long,
    ) -> bool,
}

unsafe extern "system" fn register_extension_as<T: AddInWrapper>(
    this: &mut This<1, T>,
    name: *mut *mut u16,
) -> bool {
    let component = this.get_component();
    let Some(allocator) = component.memory else {
        return false;
    };

    let extension_name = component.addin.register_extension_as();

    let Some(ptr) = allocator.alloc_str(extension_name.len()) else {
        return false;
    };
    ptr::copy_nonoverlapping(
        extension_name.as_ptr(),
        ptr.as_ptr(),
        extension_name.len(),
    );
    *name = ptr.as_ptr();

    true
}

unsafe extern "system" fn get_n_props<T: AddInWrapper>(
    this: &mut This<1, T>,
) -> c_long {
    let component = this.get_component();
    component.addin.get_n_props() as c_long
}

unsafe extern "system" fn find_prop<T: AddInWrapper>(
    this: &mut This<1, T>,
    name: *const u16,
) -> c_long {
    let component = this.get_component();
    let name = get_str(name);
    match component.addin.find_prop(name) {
        Some(i) => i as c_long,
        None => -1,
    }
}

unsafe extern "system" fn get_prop_name<T: AddInWrapper>(
    this: &mut This<1, T>,
    num: c_long,
    alias: c_long,
) -> *const u16 {
    let component = this.get_component();
    let Some(allocator) = component.memory else {
        return ptr::null();
    };
    let Some(prop_name) = component.addin.get_prop_name(num as usize, alias as usize) else {
        return ptr::null();
    };
    let Some(ptr) = allocator.alloc_str(prop_name.len()) else {
        return ptr::null();
    };
    ptr::copy_nonoverlapping(prop_name.as_ptr(), ptr.as_ptr(), prop_name.len());

    ptr.as_ptr()
}

unsafe extern "system" fn get_prop_val<T: AddInWrapper>(
    component: &mut This<1, T>,
    num: c_long,
    val: &mut TVariant,
) -> bool {
    let component = component.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let mut result = true;
    let return_value = ReturnValue {
        mem,
        variant: val,
        result: &mut result,
    };
    component.addin.get_prop_val(num as usize, return_value) && result
}

unsafe extern "system" fn set_prop_val<T: AddInWrapper>(
    this: &mut This<1, T>,
    num: c_long,
    val: &TVariant,
) -> bool {
    let component = this.get_component();
    let param = ParamValue::from(val);
    component.addin.set_prop_val(num as usize, &param)
}

unsafe extern "system" fn is_prop_readable<T: AddInWrapper>(
    this: &mut This<1, T>,
    num: c_long,
) -> bool {
    let component = this.get_component();
    component.addin.is_prop_readable(num as usize)
}

unsafe extern "system" fn is_prop_writable<T: AddInWrapper>(
    this: &mut This<1, T>,
    num: c_long,
) -> bool {
    let component = this.get_component();
    component.addin.is_prop_writable(num as usize)
}

unsafe extern "system" fn get_n_methods<T: AddInWrapper>(
    this: &mut This<1, T>,
) -> c_long {
    let component = this.get_component();
    component.addin.get_n_methods() as c_long
}

unsafe extern "system" fn find_method<T: AddInWrapper>(
    this: &mut This<1, T>,
    name: *const u16,
) -> c_long {
    let component = this.get_component();
    let name = get_str(name);
    match component.addin.find_method(name) {
        Some(i) => i as c_long,
        None => -1,
    }
}

unsafe extern "system" fn get_method_name<T: AddInWrapper>(
    this: &mut This<1, T>,
    num: c_long,
    alias: c_long,
) -> *const u16 {
    let component = this.get_component();
    let Some(allocator) = component.memory else {
        return ptr::null();
    };
    let Some(method_name) = component.addin.get_method_name(num as usize, alias as usize) else {
        return ptr::null();
    };
    let Some(ptr) = allocator.alloc_str(method_name.len()) else {
        return ptr::null();
    };

    ptr::copy_nonoverlapping(
        method_name.as_ptr(),
        ptr.as_ptr(),
        method_name.len(),
    );

    ptr.as_ptr()
}

unsafe extern "system" fn get_n_params<T: AddInWrapper>(
    this: &mut This<1, T>,
    num: c_long,
) -> c_long {
    let component = this.get_component();
    component.addin.get_n_params(num as usize) as c_long
}

unsafe extern "system" fn get_param_def_value<T: AddInWrapper>(
    this: &mut This<1, T>,
    method_num: c_long,
    param_num: c_long,
    val: &mut TVariant,
) -> bool {
    let component = this.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let mut result = true;
    let return_value = ReturnValue {
        mem,
        variant: val,
        result: &mut result,
    };

    component.addin.get_param_def_value(
        method_num as usize,
        param_num as usize,
        return_value,
    ) && result
}

unsafe extern "system" fn has_ret_val<T: AddInWrapper>(
    this: &mut This<1, T>,
    method_num: c_long,
) -> bool {
    let component = this.get_component();
    component.addin.has_ret_val(method_num as usize)
}

unsafe extern "system" fn call_as_proc<T: AddInWrapper>(
    this: &mut This<1, T>,
    method_num: c_long,
    params: *const TVariant,
    size_array: c_long,
) -> bool {
    let component = this.get_component();
    let param_values = from_raw_parts(params, size_array as usize)
        .iter()
        .map(ParamValue::from)
        .collect::<Vec<ParamValue>>();

    component
        .addin
        .call_as_proc(method_num as usize, param_values.as_slice())
}

unsafe extern "system" fn call_as_func<T: AddInWrapper>(
    this: &mut This<1, T>,
    method_num: c_long,
    ret_value: &mut TVariant,
    params: *const TVariant,
    size_array: c_long,
) -> bool {
    let component = this.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let mut result = true;
    let return_value = ReturnValue {
        mem,
        variant: ret_value,
        result: &mut result,
    };

    let param_values = from_raw_parts(params, size_array as usize)
        .iter()
        .map(ParamValue::from)
        .collect::<Vec<ParamValue>>();

    component.addin.call_as_func(
        method_num as usize,
        param_values.as_slice(),
        return_value,
    ) && result
}

impl<T: AddInWrapper> LanguageExtenderBaseVTable<T> {
    pub fn new() -> Self {
        Self {
            dtor: 0,
            #[cfg(target_family = "unix")]
            dtor2: 0,
            register_extension_as,
            get_n_props,
            find_prop,
            get_prop_name,
            get_prop_val,
            set_prop_val,
            is_prop_readable,
            is_prop_writable,
            get_n_methods,
            find_method,
            get_method_name,
            get_n_params,
            get_param_def_value,
            has_ret_val,
            call_as_proc,
            call_as_func,
        }
    }
}
