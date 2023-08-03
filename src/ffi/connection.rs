use std::ffi::{c_long, c_ushort};

use super::{types::TVariant, utils::os_string};

#[repr(C)]
struct ConnectionVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    add_error: unsafe extern "system" fn(
        &Connection,
        c_ushort,
        *const u16,
        *const u16,
        c_long,
    ) -> bool,
    read: unsafe extern "system" fn(
        &Connection,
        *mut u16,
        &mut TVariant,
        c_long,
        *mut *mut u16,
    ) -> bool,
    write: unsafe extern "system" fn(&Connection, *mut u16, &mut TVariant) -> bool,
    register_profile_as: unsafe extern "system" fn(&Connection, *mut u16) -> bool,
    set_event_buffer_depth: unsafe extern "system" fn(&Connection, c_long) -> bool,
    get_event_buffer_depth: unsafe extern "system" fn(&Connection) -> c_long,
    external_event:
        unsafe extern "system" fn(&Connection, *mut u16, *mut u16, *mut u16) -> bool,
    clean_event_buffer: unsafe extern "system" fn(&Connection),
    set_status_line: unsafe extern "system" fn(&Connection, *mut u16) -> bool,
    reset_status_line: unsafe extern "system" fn(&Connection),
}

#[repr(C)]
pub struct Connection {
    vptr1: &'static ConnectionVTable,
}

impl Connection {
    pub fn add_error(&self, code: c_ushort, source: &str, description: &str) -> bool {
        unsafe {
            let source_ptr = source.encode_utf16().collect::<Vec<u16>>().as_mut_ptr();
            let description_ptr = description
                .encode_utf16()
                .collect::<Vec<u16>>()
                .as_mut_ptr();
            (self.vptr1.add_error)(self, code, source_ptr, description_ptr, 0)
        }
    }

    pub fn external_event(&self, caller: &str, name: &str, data: &str) -> bool {
        unsafe {
            let caller_ptr = os_string(caller).as_mut_ptr();
            let name_ptr = os_string(name).as_mut_ptr();
            let data_ptr = os_string(data).as_mut_ptr();
            (self.vptr1.external_event)(self, caller_ptr, name_ptr, data_ptr)
        }
    }

    pub fn set_event_buffer_depth(&self, depth: c_long) -> bool {
        unsafe { (self.vptr1.set_event_buffer_depth)(self, depth) }
    }

    pub fn get_event_buffer_depth(&self) -> c_long {
        unsafe { (self.vptr1.get_event_buffer_depth)(self) }
    }
}
