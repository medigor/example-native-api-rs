use std::{
    ffi::{c_int, c_long, c_ulong, c_void},
    ptr::{self, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};

use smallvec::SmallVec;

#[repr(C)]
#[derive(Debug)]
pub enum AttachType {
    NotIsolated = 1,
    Isolated,
    Any,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Tm {
    pub sec: c_int,   // seconds after the minute - [0, 60] including leap second
    pub min: c_int,   // minutes after the hour - [0, 59]
    pub hour: c_int,  // hours since midnight - [0, 23]
    pub mday: c_int,  // day of the month - [1, 31]
    pub mon: c_int,   // months since January - [0, 11]
    pub year: c_int,  // years since 1900
    pub wday: c_int,  // days since Sunday - [0, 6]
    pub yday: c_int,  // days since January 1 - [0, 365]
    pub isdst: c_int, // daylight savings time flag

    #[cfg(target_family = "unix")]
    pub gmtoff: std::ffi::c_long, // seconds east of UTC
    #[cfg(target_family = "unix")]
    pub zone: std::ffi::c_char, // timezone abbreviation
}

pub struct Variant<'a> {
    mem: &'a MemoryManager,
    variant: &'a mut TVariant,
}

#[allow(dead_code)]
impl<'a> Variant<'a> {
    pub fn get(&self) -> ParamValue {
        ParamValue::from(self.variant as &_)
    }
    fn free_memory(&mut self) {
        match self.variant.vt {
            VariantType::Pwstr => unsafe {
                self.mem.free_str(&mut self.variant.value.data_str.ptr)
            },
            VariantType::Blob => unsafe {
                self.mem.free_blob(&mut self.variant.value.data_blob.ptr)
            },
            _ => (),
        }
    }
    pub fn set_empty(&mut self) {
        self.free_memory();
        self.variant.vt = VariantType::Empty;
    }
    pub fn set_i32(&mut self, val: i32) {
        self.free_memory();
        self.variant.vt = VariantType::I4;
        self.variant.value.i32 = val;
    }

    pub fn set_bool(&mut self, val: bool) {
        self.free_memory();
        self.variant.vt = VariantType::Bool;
        self.variant.value.bool = val;
    }

    pub fn set_f64(&mut self, val: f64) {
        self.free_memory();
        self.variant.vt = VariantType::R8;
        self.variant.value.f64 = val;
    }

    pub fn set_date(&mut self, val: Tm) {
        self.free_memory();
        self.variant.vt = VariantType::TM;
        self.variant.value.tm = val;
    }

    #[must_use]
    pub fn set_str(&mut self, val: &[u16]) -> bool {
        let Some(ptr) = self.mem.alloc_str(val.len()) else {
            return false;
        };
        self.free_memory();

        unsafe { ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr(), val.len()) };

        self.variant.vt = VariantType::Pwstr;
        self.variant.value.data_str.ptr = ptr.as_ptr();
        self.variant.value.data_str.len = val.len() as u32;
        true
    }

    #[must_use]
    pub fn set_blob(&mut self, val: &[u8]) -> bool {
        let Some(ptr) = self.mem.alloc_blob(val.len()) else {
            return false;
        };
        self.free_memory();

        unsafe { ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr(), val.len()) };

        self.variant.vt = VariantType::Blob;
        self.variant.value.data_blob.ptr = ptr.as_ptr();
        self.variant.value.data_blob.len = val.len() as u32;
        true
    }

    pub fn alloc_str(&mut self, len: usize) -> Option<&'a mut [u16]> {
        let Some(ptr) = self.mem.alloc_str(len) else {
            return None;
        };

        self.free_memory();

        self.variant.vt = VariantType::Pwstr;
        self.variant.value.data_str.ptr = ptr.as_ptr();
        self.variant.value.data_str.len = len as u32;

        Some(unsafe { from_raw_parts_mut(ptr.as_ptr(), len) })
    }

    pub fn alloc_blob(&mut self, len: usize) -> Option<&'a mut [u8]> {
        let Some(ptr) = self.mem.alloc_blob(len) else {
            return None;
        };
        self.free_memory();

        self.variant.vt = VariantType::Blob;
        self.variant.value.data_blob.ptr = ptr.as_ptr();
        self.variant.value.data_blob.len = len as u32;

        Some(unsafe { from_raw_parts_mut(ptr.as_ptr(), len) })
    }
}

pub enum ParamValue<'a> {
    Empty,
    Bool(bool),
    I32(i32),
    F64(f64),
    Date(Tm),
    Str(&'a [u16]),
    Blob(&'a [u8]),
}

impl<'a> From<&'a TVariant> for ParamValue<'a> {
    fn from(param: &'a TVariant) -> ParamValue {
        unsafe {
            match param.vt {
                VariantType::Empty => Self::Empty,
                VariantType::Bool => Self::Bool(param.value.bool),
                VariantType::I4 => Self::I32(param.value.i32),
                VariantType::R8 => Self::F64(param.value.f64),
                VariantType::TM => Self::Date(param.value.tm),
                VariantType::Pwstr => Self::Str(from_raw_parts(
                    param.value.data_str.ptr,
                    param.value.data_str.len as usize,
                )),
                VariantType::Blob => Self::Blob(from_raw_parts(
                    param.value.data_blob.ptr,
                    param.value.data_blob.len as usize,
                )),
                _ => Self::Empty,
            }
        }
    }
}

#[repr(u16)]
#[allow(dead_code)]
enum VariantType {
    Empty = 0,
    Null,
    I2,        //int16_t
    I4,        //int32_t
    R4,        //float
    R8,        //double
    Date,      //DATE (double)
    TM,        //struct tm
    Pstr,      //struct str    string
    Interface, //struct iface
    Error,     //int32_t errCode
    Bool,      //bool
    Variant,   //struct _tVariant *
    I1,        //int8_t
    Ui1,       //uint8_t
    Ui2,       //uint16_t
    Ui4,       //uint32_t
    I8,        //int64_t
    Ui8,       //uint64_t
    Int,       //int   Depends on architecture
    Uint,      //unsigned int  Depends on architecture
    Hresult,   //long hRes
    Pwstr,     //struct wstr
    Blob,      //means in struct str binary data contain
    Clsid,     //UUID

    Undefined = 0xFFFF,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DataStr {
    pub ptr: *mut u16,
    pub len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DataBlob {
    pub ptr: *mut u8,
    pub len: u32,
}

#[repr(C)]
union VariantValue {
    pub bool: bool,
    pub i32: i32,
    pub f64: f64,
    pub tm: Tm,
    pub data_str: DataStr,
    pub data_blob: DataBlob,
}

#[repr(C)]
struct TVariant {
    value: VariantValue,
    elements: u32, //Dimension for an one-dimensional array in pvarVal
    vt: VariantType,
}

#[allow(unused_variables)]
pub trait Addin {
    fn init(&mut self, interface: &'static Connection) -> bool {
        true
    }

    /// default 2000, don't use version 1000, because static objects are created
    fn get_info(&mut self) -> u16 {
        2000
    }
    fn done(&mut self) {}
    fn register_extension_as(&mut self) -> &'static [u16];
    fn get_n_props(&mut self) -> usize {
        0
    }
    fn find_prop(&mut self, name: &[u16]) -> Option<usize> {
        None
    }
    fn get_prop_name(&mut self, num: usize, alias: usize) -> Option<&'static [u16]> {
        None
    }
    fn get_prop_val(&mut self, num: usize, val: &mut Variant) -> bool {
        false
    }
    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool {
        false
    }
    fn is_prop_readable(&mut self, num: usize) -> bool {
        false
    }
    fn is_prop_writable(&mut self, num: usize) -> bool {
        false
    }
    fn get_n_methods(&mut self) -> usize {
        0
    }
    fn find_method(&mut self, name: &[u16]) -> Option<usize> {
        None
    }
    fn get_method_name(&mut self, num: usize, alias: usize) -> Option<&'static [u16]> {
        None
    }
    fn get_n_params(&mut self, num: usize) -> usize {
        0
    }
    fn get_param_def_value(&mut self, method_num: usize, param_num: usize, value: Variant) -> bool {
        true
    }
    fn has_ret_val(&mut self, method_num: usize) -> bool {
        false
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
        false
    }
    fn set_locale(&mut self, loc: &[u16]) {}
    fn set_user_interface_language_code(&mut self, lang: &[u16]) {}
}

#[repr(C)]
struct This<const OFFSET: usize, T: Addin> {
    ptr: *mut Component<T>,
}

impl<'a, const OFFSET: usize, T: Addin> This<OFFSET, T> {
    unsafe fn get_component(&mut self) -> &'a mut Component<T> {
        let new_ptr = (self as *mut This<OFFSET, T> as *mut c_void)
            .sub(OFFSET * std::mem::size_of::<usize>());
        &mut *(new_ptr as *mut Component<T>)
    }
}

#[repr(C)]
struct InitDoneBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    init: unsafe extern "system" fn(&mut This<0, T>, &'static Connection) -> bool,
    set_mem_manager: unsafe extern "system" fn(&mut This<0, T>, &'static MemoryManager) -> bool,
    get_info: unsafe extern "system" fn(&mut This<0, T>) -> c_long,
    done: unsafe extern "system" fn(&mut This<0, T>),
}

unsafe extern "system" fn init<T: Addin>(
    this: &mut This<0, T>,
    interface: &'static Connection,
) -> bool {
    let component = this.get_component();
    component.addin.init(interface)
}

unsafe extern "system" fn set_mem_manager<T: Addin>(
    this: &mut This<0, T>,
    mem: &'static MemoryManager,
) -> bool {
    let component = this.get_component();
    component.memory = Some(mem);
    true
}

unsafe extern "system" fn get_info<T: Addin>(this: &mut This<0, T>) -> c_long {
    let component = this.get_component();
    component.addin.get_info() as c_long
}

unsafe extern "system" fn done<T: Addin>(this: &mut This<0, T>) {
    let component = this.get_component();
    component.addin.done()
}

#[repr(C)]
struct LanguageExtenderBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    register_extension_as: unsafe extern "system" fn(&mut This<1, T>, *mut *mut u16) -> bool,
    get_n_props: unsafe extern "system" fn(&mut This<1, T>) -> c_long,
    find_prop: unsafe extern "system" fn(&mut This<1, T>, *const u16) -> c_long,
    get_prop_name: unsafe extern "system" fn(&mut This<1, T>, c_long, c_long) -> *const u16,
    get_prop_val: unsafe extern "system" fn(&mut This<1, T>, c_long, &mut TVariant) -> bool,
    set_prop_val: unsafe extern "system" fn(&mut This<1, T>, c_long, &TVariant) -> bool,
    is_prop_readable: unsafe extern "system" fn(&mut This<1, T>, c_long) -> bool,
    is_prop_writable: unsafe extern "system" fn(&mut This<1, T>, c_long) -> bool,
    get_n_methods: unsafe extern "system" fn(&mut This<1, T>) -> c_long,
    find_method: unsafe extern "system" fn(&mut This<1, T>, *const u16) -> c_long,
    get_method_name: unsafe extern "system" fn(&mut This<1, T>, c_long, c_long) -> *const u16,
    get_n_params: unsafe extern "system" fn(&mut This<1, T>, c_long) -> c_long,
    get_param_def_value:
        unsafe extern "system" fn(&mut This<1, T>, c_long, c_long, &mut TVariant) -> bool,
    has_ret_val: unsafe extern "system" fn(&mut This<1, T>, c_long) -> bool,
    call_as_proc: unsafe extern "system" fn(&mut This<1, T>, c_long, *mut TVariant, c_long) -> bool,
    call_as_func: unsafe extern "system" fn(
        &mut This<1, T>,
        c_long,
        &mut TVariant,
        *mut TVariant,
        c_long,
    ) -> bool,
}

unsafe extern "system" fn register_extension_as<T: Addin>(
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
    ptr::copy_nonoverlapping(extension_name.as_ptr(), ptr.as_ptr(), extension_name.len());
    *name = ptr.as_ptr();

    true
}

unsafe extern "system" fn get_n_props<T: Addin>(this: &mut This<1, T>) -> c_long {
    let component = this.get_component();
    component.addin.get_n_props() as c_long
}

unsafe extern "system" fn find_prop<T: Addin>(this: &mut This<1, T>, name: *const u16) -> c_long {
    let component = this.get_component();
    let name = get_str(name);
    match component.addin.find_prop(name) {
        Some(i) => i as c_long,
        None => -1,
    }
}

unsafe extern "system" fn get_prop_name<T: Addin>(
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

unsafe extern "system" fn get_prop_val<T: Addin>(
    component: &mut This<1, T>,
    num: c_long,
    val: &mut TVariant,
) -> bool {
    let component = component.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let mut return_value = Variant { mem, variant: val };
    component
        .addin
        .get_prop_val(num as usize, &mut return_value)
}

unsafe extern "system" fn set_prop_val<T: Addin>(
    this: &mut This<1, T>,
    num: c_long,
    val: &TVariant,
) -> bool {
    let component = this.get_component();
    let param = ParamValue::from(val);
    component.addin.set_prop_val(num as usize, &param)
}

unsafe extern "system" fn is_prop_readable<T: Addin>(this: &mut This<1, T>, num: c_long) -> bool {
    let component = this.get_component();
    component.addin.is_prop_readable(num as usize)
}

unsafe extern "system" fn is_prop_writable<T: Addin>(this: &mut This<1, T>, num: c_long) -> bool {
    let component = this.get_component();
    component.addin.is_prop_writable(num as usize)
}

unsafe extern "system" fn get_n_methods<T: Addin>(this: &mut This<1, T>) -> c_long {
    let component = this.get_component();
    component.addin.get_n_methods() as c_long
}

unsafe extern "system" fn find_method<T: Addin>(this: &mut This<1, T>, name: *const u16) -> c_long {
    let component = this.get_component();
    let name = get_str(name);
    match component.addin.find_method(name) {
        Some(i) => i as c_long,
        None => -1,
    }
}

unsafe extern "system" fn get_method_name<T: Addin>(
    this: &mut This<1, T>,
    num: c_long,
    alias: c_long,
) -> *const u16 {
    let component = this.get_component();
    let Some(allocator) = component.memory else {
        return ptr::null();
    };
    let Some(method_name) = component
        .addin
        .get_method_name(num as usize, alias as usize)
    else {
        return ptr::null();
    };
    let Some(ptr) = allocator.alloc_str(method_name.len()) else {
        return ptr::null();
    };

    ptr::copy_nonoverlapping(method_name.as_ptr(), ptr.as_ptr(), method_name.len());

    ptr.as_ptr()
}

unsafe extern "system" fn get_n_params<T: Addin>(this: &mut This<1, T>, num: c_long) -> c_long {
    let component = this.get_component();
    component.addin.get_n_params(num as usize) as _
}

unsafe extern "system" fn get_param_def_value<T: Addin>(
    this: &mut This<1, T>,
    method_num: c_long,
    param_num: c_long,
    val: &mut TVariant,
) -> bool {
    let component = this.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let return_value = Variant { mem, variant: val };

    component
        .addin
        .get_param_def_value(method_num as usize, param_num as usize, return_value)
}

unsafe extern "system" fn has_ret_val<T: Addin>(this: &mut This<1, T>, method_num: c_long) -> bool {
    let component = this.get_component();
    component.addin.has_ret_val(method_num as usize)
}

unsafe extern "system" fn call_as_proc<T: Addin>(
    this: &mut This<1, T>,
    method_num: c_long,
    params: *mut TVariant,
    size_array: c_long,
) -> bool {
    let component = this.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let size_array = size_array as usize;

    let mut param_values = SmallVec::<[Variant; 8]>::new();
    for variant in from_raw_parts_mut(params, size_array) {
        param_values.push(Variant { mem, variant });
    }

    component
        .addin
        .call_as_proc(method_num as usize, &mut param_values)
}

unsafe extern "system" fn call_as_func<T: Addin>(
    this: &mut This<1, T>,
    method_num: c_long,
    ret_value: &mut TVariant,
    params: *mut TVariant,
    size_array: c_long,
) -> bool {
    let component = this.get_component();
    let Some(mem) = component.memory else {
        return false;
    };

    let size_array = size_array as usize;

    let mut return_value = Variant {
        mem,
        variant: ret_value,
    };

    let mut param_values = SmallVec::<[Variant; 8]>::new();
    for variant in from_raw_parts_mut(params, size_array) {
        param_values.push(Variant { mem, variant });
    }

    component
        .addin
        .call_as_func(method_num as usize, &mut param_values, &mut return_value)
}

#[repr(C)]
struct LocaleBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_locale: unsafe extern "system" fn(&mut This<2, T>, *const u16),
}

unsafe extern "system" fn set_locale<T: Addin>(this: &mut This<2, T>, loc: *const u16) {
    let component = this.get_component();
    let loc = get_str(loc);
    component.addin.set_locale(loc)
}

#[repr(C)]
struct UserLanguageBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_user_interface_language_code: unsafe extern "system" fn(&mut This<3, T>, *const u16),
}

unsafe extern "system" fn set_user_interface_language_code<T: Addin>(
    this: &mut This<3, T>,
    lang: *const u16,
) {
    let component = this.get_component();
    let lang = get_str(lang);
    component.addin.set_user_interface_language_code(lang)
}

#[repr(C)]
struct Component<T: Addin> {
    vptr1: Box<InitDoneBaseVTable<T>>,
    vptr2: Box<LanguageExtenderBaseVTable<T>>,
    vptr3: Box<LocaleBaseVTable<T>>,
    vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut Component<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

unsafe extern "system" fn destroy<T: Addin>(component: *mut *mut Component<T>) {
    let comp = Box::from_raw(*component);
    drop(comp);
}

pub unsafe fn create_component<T: Addin>(component: *mut *mut c_void, addin: T) -> c_long {
    let vptr1 = Box::new(InitDoneBaseVTable {
        dtor: 0,
        #[cfg(target_family = "unix")]
        dtor2: 0,
        init,
        set_mem_manager,
        get_info,
        done,
    });

    let vptr2 = Box::new(LanguageExtenderBaseVTable {
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
    });

    let vptr3 = Box::new(LocaleBaseVTable {
        dtor: 0,
        #[cfg(target_family = "unix")]
        dtor2: 0,
        set_locale,
    });

    let vptr4 = Box::new(UserLanguageBaseVTable {
        dtor: 0,
        #[cfg(target_family = "unix")]
        dtor2: 0,
        set_user_interface_language_code,
    });

    let c = Box::new(Component {
        vptr1,
        vptr2,
        vptr3,
        vptr4,
        destroy: destroy::<T>,
        memory: None,
        addin,
    });

    *component = Box::into_raw(c) as *mut c_void;
    1
}

pub unsafe fn destroy_component(component: *mut *mut c_void) -> c_long {
    #[repr(C)]
    struct ComponentWrapper {
        vptr1: usize,
        vptr2: usize,
        vptr3: usize,
        vptr4: usize,
        destroy: unsafe extern "system" fn(*mut *mut c_void),
    }

    let wrapper = *component as *mut ComponentWrapper;
    let wrapper = &mut *wrapper;
    (wrapper.destroy)(component);
    *component = ptr::null_mut();

    0
}

#[repr(C)]
struct MemoryManagerVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    alloc_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void, c_ulong) -> bool,
    free_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void),
}

#[repr(C)]
struct MemoryManager {
    vptr: &'static MemoryManagerVTable,
}

impl MemoryManager {
    pub fn alloc_blob(&self, size: usize) -> Option<NonNull<u8>> {
        let mut ptr = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr.alloc_memory)(self, &mut ptr, size as c_ulong) {
                NonNull::new(ptr as *mut u8)
            } else {
                None
            }
        }
    }

    pub fn alloc_str(&self, size: usize) -> Option<NonNull<u16>> {
        let mut ptr = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr.alloc_memory)(self, &mut ptr, size as c_ulong * 2) {
                NonNull::new(ptr as *mut u16)
            } else {
                None
            }
        }
    }

    pub fn free_str(&self, ptr: *mut *mut u16) {
        unsafe {
            (self.vptr.free_memory)(self, ptr as _);
        }
    }

    pub fn free_blob(&self, ptr: *mut *mut u8) {
        unsafe {
            (self.vptr.free_memory)(self, ptr as _);
        }
    }
}

#[repr(C)]
struct ConnectionVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
}

#[repr(C)]
pub struct Connection {
    vptr1: &'static ConnectionVTable,
}

unsafe fn get_str<'a>(s: *const u16) -> &'a [u16] {
    unsafe fn strlen(s: *const u16) -> usize {
        let mut i = 0;
        while *s.add(i) != 0 {
            i += 1;
        }
        i + 1
    }

    let len = strlen(s);
    from_raw_parts(s, len)
}
