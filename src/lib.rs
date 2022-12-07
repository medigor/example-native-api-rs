mod addin1;
mod ffi;

use std::{
    ffi::{c_long, c_void, c_int},
    sync::atomic::{AtomicI32, Ordering},
};

use addin1::Addin1;
use ffi::{destroy_component, AttachType};
use utf16_lit::utf16_null;

use crate::ffi::create_component;

pub static mut PLATFORM_CAPABILITIES: AtomicI32 = AtomicI32::new(-1);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetClassObject(_name: *const u16, component: *mut *mut c_void) -> c_long {
    let addin = Addin1::new();
    unsafe {
        *component = create_component(addin);
    }
    1
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn DestroyObject(component: *mut *mut c_void) -> c_long {
    destroy_component(component);
    0
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetClassNames() -> *const u16 {
    utf16_null!("Class1").as_ptr()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn SetPlatformCapabilities(capabilities: c_int) -> c_long {
    unsafe {
        PLATFORM_CAPABILITIES.store(capabilities, Ordering::Relaxed);
    }
    3
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetAttachType() -> AttachType {
    AttachType::CanAttachAny
}
