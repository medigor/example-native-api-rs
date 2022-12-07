use std::{
    ffi::{c_int, c_long, c_ulong, c_void},
    mem::size_of,
    ptr,
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(C)]
#[derive(Debug)]
pub enum AttachType {
    CanAttachNotIsolated = 1,
    CanAttachIsolated,
    CanAttachAny,
}

#[repr(C)]
#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct ReturnValue<'a> {
    mem: &'a MemoryManager,
    variant: &'a mut TVariant,
}

#[allow(dead_code)]
impl<'a> ReturnValue<'a> {
    pub fn set_empty(self) {
        self.variant.vt = VariantType::EMPTY;
    }
    pub fn set_i32(self, val: i32) {
        self.variant.vt = VariantType::I4;
        self.variant.value.i32 = val;
    }

    pub fn set_bool(self, val: bool) {
        self.variant.vt = VariantType::BOOL;
        self.variant.value.bool = val;
    }

    pub fn set_f64(self, val: f64) {
        self.variant.vt = VariantType::R8;
        self.variant.value.f64 = val;
    }

    pub fn set_date(self, val: Tm) {
        self.variant.vt = VariantType::TM;
        self.variant.value.tm = val;
    }

    pub fn set_str(self, val: &[u16]) {
        let Some(data) = self.mem.alloc_memory::<u16>(val.len()) else {
            return;
        };

        data.copy_from_slice(val);

        self.variant.vt = VariantType::PWSTR;
        self.variant.value.data_str.ptr = data.as_ptr() as *mut u16;
        self.variant.value.data_str.len = data.len() as u32;
    }

    pub fn set_blob(self, val: &[u8]) {
        let Some(data) = self.mem.alloc_memory::<u8>(val.len()) else {
            return;
        };

        data.copy_from_slice(val);

        self.variant.vt = VariantType::BLOB;
        self.variant.value.data_blob.ptr = data.as_ptr() as *mut u8;
        self.variant.value.data_blob.len = data.len() as u32;
    }

    pub fn alloc_str(self, len: usize) -> Option<&'a mut [u16]> {
        let Some(data) = self.mem.alloc_memory::<u16>(len) else {
            return None;
        };

        self.variant.vt = VariantType::PWSTR;
        self.variant.value.data_str.ptr = data.as_ptr() as *mut u16;
        self.variant.value.data_str.len = data.len() as u32;

        Some(data)
    }

    pub fn alloc_blob(self, len: usize) -> Option<&'a mut [u8]> {
        let Some(data) = self.mem.alloc_memory::<u8>(len) else {
            return None;
        };

        self.variant.vt = VariantType::BLOB;
        self.variant.value.data_blob.ptr = data.as_ptr() as *mut u8;
        self.variant.value.data_blob.len = data.len() as u32;

        Some(data)
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
                VariantType::EMPTY => Self::Empty,
                VariantType::BOOL => Self::Bool(param.value.bool),
                VariantType::I4 => Self::I32(param.value.i32),
                VariantType::R8 => Self::F64(param.value.f64),
                VariantType::TM => Self::Date(param.value.tm),
                VariantType::PWSTR => Self::Str(from_raw_parts(
                    param.value.data_str.ptr,
                    param.value.data_str.len as usize,
                )),
                VariantType::BLOB => Self::Blob(from_raw_parts(
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
    EMPTY = 0,
    NULL,
    I2,        //int16_t
    I4,        //int32_t
    R4,        //float
    R8,        //double
    DATE,      //DATE (double)
    TM,        //struct tm
    PSTR,      //struct str    string
    INTERFACE, //struct iface
    ERROR,     //int32_t errCode
    BOOL,      //bool
    VARIANT,   //struct _tVariant *
    I1,        //int8_t
    UI1,       //uint8_t
    UI2,       //uint16_t
    UI4,       //uint32_t
    I8,        //int64_t
    UI8,       //uint64_t
    INT,       //int   Depends on architecture
    UINT,      //unsigned int  Depends on architecture
    HRESULT,   //long hRes
    PWSTR,     //struct wstr
    BLOB,      //means in struct str binary data contain
    CLSID,     //UUID

    UNDEFINED = 0xFFFF,
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
struct DataStr {
    pub ptr: *mut u16,
    pub len: u32,
}

#[repr(C)]
#[allow(dead_code)]
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
#[allow(dead_code)]
struct TVariant {
    value: VariantValue,
    elements: u32, //Dimension for an one-dimensional array in pvarVal
    vt: VariantType,
}

pub trait Addin {
    fn init(&mut self, interface: &'static Connection) -> bool;
    fn get_info(&mut self) -> u16;
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

#[repr(C)]
//#[allow(dead_code)]
struct InitDoneBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    init: unsafe extern "system" fn(&mut InitDoneBase<T>, &'static Connection) -> bool,
    set_mem_manager:
        unsafe extern "system" fn(&mut InitDoneBase<T>, &'static MemoryManager) -> bool,
    get_info: unsafe extern "system" fn(&mut InitDoneBase<T>) -> c_long,
    done: unsafe extern "system" fn(&mut InitDoneBase<T>),
}

unsafe extern "system" fn init<T: Addin>(
    component: &mut InitDoneBase<T>,
    interface: &'static Connection,
) -> bool {
    component.addin.init(interface)
}

unsafe extern "system" fn set_mem_manager<T: Addin>(
    component: &mut InitDoneBase<T>,
    mem: &'static MemoryManager,
) -> bool {
    component.memory = Some(mem);
    true
}

unsafe extern "system" fn get_info<T: Addin>(component: &mut InitDoneBase<T>) -> c_long {
    component.addin.get_info() as c_long
}

unsafe extern "system" fn done<T: Addin>(component: &mut InitDoneBase<T>) {
    component.addin.done()
}

#[repr(C)]
#[allow(dead_code)]
struct LanguageExtenderBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    register_extension_as:
        unsafe extern "system" fn(&mut LanguageExtenderBase<T>, *mut *mut u16) -> bool,
    get_n_props: unsafe extern "system" fn(&mut LanguageExtenderBase<T>) -> c_long,
    find_prop: unsafe extern "system" fn(&mut LanguageExtenderBase<T>, *const u16) -> c_long,
    get_prop_name:
        unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long, c_long) -> *const u16,
    get_prop_val: for<'a> unsafe extern "system" fn(
        &mut LanguageExtenderBase<T>,
        c_long,
        &'a mut TVariant,
    ) -> bool,
    set_prop_val:
        unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long, &TVariant) -> bool,
    is_prop_readable: unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long) -> bool,
    is_prop_writable: unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long) -> bool,
    get_n_methods: unsafe extern "system" fn(&mut LanguageExtenderBase<T>) -> c_long,
    find_method: unsafe extern "system" fn(&mut LanguageExtenderBase<T>, *const u16) -> c_long,
    get_method_name:
        unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long, c_long) -> *const u16,
    get_n_params: unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long) -> c_long,
    get_param_def_value: unsafe extern "system" fn(
        &mut LanguageExtenderBase<T>,
        c_long,
        c_long,
        &mut TVariant,
    ) -> bool,
    has_ret_val: unsafe extern "system" fn(&mut LanguageExtenderBase<T>, c_long) -> bool,
    call_as_proc: unsafe extern "system" fn(
        &mut LanguageExtenderBase<T>,
        c_long,
        *const TVariant,
        c_long,
    ) -> bool,
    call_as_func: for<'a> unsafe extern "system" fn(
        &mut LanguageExtenderBase<T>,
        c_long,
        &mut TVariant,
        *const TVariant,
        c_long,
    ) -> bool,
}

unsafe extern "system" fn register_extension_as<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    name: *mut *mut u16,
) -> bool {
    let Some(allocator) = component.memory else {
        return false;
    };

    let extension_name = component.addin.register_extension_as();

    let Some(data) = allocator.alloc_memory::<u16>(extension_name.len()) else {
        return false;
    };
    data.copy_from_slice(extension_name);
    unsafe { *name = data.as_mut_ptr() };

    true
}

unsafe extern "system" fn get_n_props<T: Addin>(component: &mut LanguageExtenderBase<T>) -> c_long {
    component.addin.get_n_props() as c_long
}

unsafe extern "system" fn find_prop<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    name: *const u16,
) -> c_long {
    let len = strlen(name);
    let name = from_raw_parts(name, len);
    match component.addin.find_prop(name) {
        Some(i) => i as c_long,
        None => -1,
    }
}

unsafe extern "system" fn get_prop_name<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
    alias: c_long,
) -> *const u16 {
    let Some(allocator) = component.memory else {
        return ptr::null();
    };
    let Some(prop_name) = component.addin.get_prop_name(num as usize, alias as usize) else {
        return ptr::null();
    };
    let Some(name) = allocator.alloc_memory::<u16>(prop_name.len()) else {
        return ptr::null();
    };

    name.copy_from_slice(prop_name);
    name.as_ptr()
}

unsafe extern "system" fn get_prop_val<'a, T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
    val: &'a mut TVariant,
) -> bool {
    let Some(mem) = component.memory else {
        return false;
    };

    let return_value = ReturnValue { mem, variant: val };
    component.addin.get_prop_val(num as usize, return_value)
}

unsafe extern "system" fn set_prop_val<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
    val: &TVariant,
) -> bool {
    let param = ParamValue::from(val);
    component.addin.set_prop_val(num as usize, &param)
}

unsafe extern "system" fn is_prop_readable<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
) -> bool {
    component.addin.is_prop_readable(num as usize)
}

unsafe extern "system" fn is_prop_writable<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
) -> bool {
    component.addin.is_prop_writable(num as usize)
}

unsafe extern "system" fn get_n_methods<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
) -> c_long {
    component.addin.get_n_methods() as c_long
}

unsafe extern "system" fn find_method<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    name: *const u16,
) -> c_long {
    let len = strlen(name);
    let name = from_raw_parts(name, len);
    match component.addin.find_method(name) {
        Some(i) => i as c_long,
        None => -1,
    }
}

unsafe extern "system" fn get_method_name<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
    alias: c_long,
) -> *const u16 {
    let Some(allocator) = component.memory else {
        return ptr::null();
    };
    let Some(method_name) = component.addin.get_method_name(num as usize, alias as usize) else {
        return ptr::null();
    };
    let Some(name) = allocator.alloc_memory::<u16>(method_name.len()) else {
        return ptr::null();
    };

    name.copy_from_slice(method_name);
    name.as_ptr()
}

unsafe extern "system" fn get_n_params<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    num: c_long,
) -> c_long {
    component.addin.get_n_params(num as usize) as c_long
}

unsafe extern "system" fn get_param_def_value<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    method_num: c_long,
    param_num: c_long,
    val: &mut TVariant,
) -> bool {
    let Some(mem) = component.memory else {
        return false;
    };

    let return_value = ReturnValue { mem, variant: val };

    component
        .addin
        .get_param_def_value(method_num as usize, param_num as usize, return_value)
}

unsafe extern "system" fn has_ret_val<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    method_num: c_long,
) -> bool {
    component.addin.has_ret_val(method_num as usize)
}

unsafe extern "system" fn call_as_proc<T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    method_num: c_long,
    params: *const TVariant,
    size_array: c_long,
) -> bool {
    let param_values = from_raw_parts(params, size_array as usize)
        .iter()
        .map(|x| ParamValue::from(x))
        .collect::<Vec<ParamValue>>();

    component
        .addin
        .call_as_proc(method_num as usize, param_values.as_slice())
}

unsafe extern "system" fn call_as_func<'a, T: Addin>(
    component: &mut LanguageExtenderBase<T>,
    method_num: c_long,
    ret_value: &'a mut TVariant,
    params: *const TVariant,
    size_array: c_long,
) -> bool {
    let Some(mem) = component.memory else {
        return false;
    };

    let return_value = ReturnValue {
        mem,
        variant: ret_value,
    };

    let param_values = from_raw_parts(params, size_array as usize)
        .iter()
        .map(|x| ParamValue::from(x))
        .collect::<Vec<ParamValue>>();

    component
        .addin
        .call_as_func(method_num as usize, param_values.as_slice(), return_value)
}

#[repr(C)]
#[allow(dead_code)]
struct LocaleBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_locale: unsafe extern "system" fn(&mut LocaleBase<T>, *const u16),
}

unsafe extern "system" fn set_locale<T: Addin>(component: &mut LocaleBase<T>, loc: *const u16) {
    let len = strlen(loc);
    let loc = from_raw_parts(loc, len);
    component.addin.set_locale(loc)
}

#[repr(C)]
#[allow(dead_code)]
struct UserLanguageBaseVTable<T: Addin> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_user_interface_language_code:
        unsafe extern "system" fn(&mut UserLanguageBase<T>, *const u16),
}

unsafe extern "system" fn set_user_interface_language_code<T: Addin>(
    component: &mut UserLanguageBase<T>,
    lang: *const u16,
) {
    let len = strlen(lang);
    let lang = from_raw_parts(lang, len);
    component.addin.set_user_interface_language_code(lang)
}

#[repr(C)]
#[allow(dead_code)]
struct ComponentBase<T: Addin> {
    vptr1: Box<InitDoneBaseVTable<T>>,
    vptr2: Box<LanguageExtenderBaseVTable<T>>,
    vptr3: Box<LocaleBaseVTable<T>>,
    vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut ComponentBase<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

unsafe extern "system" fn destroy<T: Addin>(component: *mut *mut ComponentBase<T>) {
    let component = unsafe { Box::from_raw(*component) };
    drop(component);
}

#[repr(C)]
#[allow(dead_code)]
struct InitDoneBase<T: Addin> {
    _vptr1: Box<InitDoneBaseVTable<T>>,
    _vptr2: Box<LanguageExtenderBaseVTable<T>>,
    _vptr3: Box<LocaleBaseVTable<T>>,
    _vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut ComponentBase<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

// type InitDoneBase<T> = ComponentBase<T>;

#[repr(C)]
#[allow(dead_code)]
struct LanguageExtenderBase<T: Addin> {
    _vptr2: Box<LanguageExtenderBaseVTable<T>>,
    _vptr3: Box<LocaleBaseVTable<T>>,
    _vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut ComponentBase<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

#[repr(C)]
#[allow(dead_code)]
struct LocaleBase<T: Addin> {
    _vptr3: Box<LocaleBaseVTable<T>>,
    _vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut ComponentBase<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

#[repr(C)]
#[allow(dead_code)]
struct UserLanguageBase<T: Addin> {
    _vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut ComponentBase<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

pub fn create_component<T: Addin>(addin: T) -> *mut c_void {
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

    let c = Box::new(ComponentBase {
        vptr1,
        vptr2,
        vptr3,
        vptr4,
        destroy: destroy::<T>,
        memory: None,
        addin,
    });

    let p = Box::leak(c);
    p as *mut ComponentBase<T> as *mut c_void
}

pub fn destroy_component(component: *mut *mut c_void) {
    #[repr(C)]
    #[allow(dead_code)]
    struct ComponentWrapper {
        vptr1: Box<c_void>,
        vptr2: Box<c_void>,
        vptr3: Box<c_void>,
        vptr4: Box<c_void>,
        destroy: unsafe extern "system" fn(*mut *mut c_void),
    }

    unsafe {
        let wrapper = *component as *mut ComponentWrapper;
        let wrapper = &mut *wrapper;
        (wrapper.destroy)(component);
    }
}

#[repr(C)]
#[allow(dead_code)]
struct MemoryManagerVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    alloc_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void, c_ulong) -> bool,
    free_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void),
}

#[repr(C)]
#[allow(dead_code)]
struct MemoryManager {
    vptr1: &'static MemoryManagerVTable,
}

impl MemoryManager {
    pub fn alloc_memory<'a, T>(&self, size: usize) -> Option<&'a mut [T]> {
        let mut data = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr1.alloc_memory)(self, &mut data, (size * size_of::<T>()) as c_ulong) {
                let d = from_raw_parts_mut(data as *mut T, size);
                Some(d)
            } else {
                None
            }
        }
    }
}

#[repr(C)]
#[allow(dead_code)]
struct ConnectionVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
}

#[repr(C)]
#[allow(dead_code)]
pub struct Connection {
    vptr1: &'static ConnectionVTable,
}

fn strlen(s: *const u16) -> usize {
    let mut i = 0;
    while unsafe { *s.add(i) } != 0 {
        i += 1;
    }
    i += 1;
    i
}
