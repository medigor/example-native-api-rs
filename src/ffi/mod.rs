use std::{
    ffi::{c_long, c_void},
    ptr,
};

use crate::add_in::AddInWrapper;

use self::{
    init_base::InitDoneBaseVTable, lang_extender::LanguageExtenderBaseVTable,
    memory_manager::MemoryManager, utils::get_str,
};

pub mod connection;
pub mod init_base;
pub mod lang_extender;
pub mod memory_manager;
pub mod types;
pub mod utils;

#[repr(C)]
#[derive(Debug)]
pub enum AttachType {
    NotIsolated = 1,
    Isolated,
    Any,
}

#[repr(C)]
struct This<const OFFSET: usize, T: AddInWrapper> {
    ptr: *mut Component<T>,
}

impl<'a, const OFFSET: usize, T: AddInWrapper> This<OFFSET, T> {
    unsafe fn get_component(&mut self) -> &'a mut Component<T> {
        let new_ptr = (self as *mut This<OFFSET, T> as *mut c_void)
            .sub(OFFSET * std::mem::size_of::<usize>());
        &mut *(new_ptr as *mut Component<T>)
    }
}

#[repr(C)]
struct LocaleBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_locale: unsafe extern "system" fn(&mut This<2, T>, *const u16),
}

unsafe extern "system" fn set_locale<T: AddInWrapper>(this: &mut This<2, T>, loc: *const u16) {
    let component = this.get_component();
    let loc = get_str(loc);
    component.addin.set_locale(loc)
}

#[repr(C)]
struct UserLanguageBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_user_interface_language_code: unsafe extern "system" fn(&mut This<3, T>, *const u16),
}

unsafe extern "system" fn set_user_interface_language_code<T: AddInWrapper>(
    this: &mut This<3, T>,
    lang: *const u16,
) {
    let component = this.get_component();
    let lang = get_str(lang);
    component.addin.set_user_interface_language_code(lang)
}

#[repr(C)]
struct Component<T: AddInWrapper> {
    vptr1: Box<InitDoneBaseVTable<T>>,
    vptr2: Box<LanguageExtenderBaseVTable<T>>,
    vptr3: Box<LocaleBaseVTable<T>>,
    vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut Component<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

unsafe extern "system" fn destroy<T: AddInWrapper>(component: *mut *mut Component<T>) {
    let comp = Box::from_raw(*component);
    drop(comp);
}

pub unsafe fn create_component<T: AddInWrapper>(component: *mut *mut c_void, addin: T) -> c_long {
    let vptr1 = Box::new(InitDoneBaseVTable::new());
    let vptr2 = Box::new(LanguageExtenderBaseVTable::new());
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
