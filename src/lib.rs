pub mod add_in;
mod ffi;

use std::{
    ffi::{c_int, c_long, c_void},
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use add_in::{AddIn, AddInContainer, ComponentFuncDescription, ComponentPropDescription};
use ffi::{connection::Connection, destroy_component, types::ParamValue, AttachType};
use utf16_lit::utf16_null;

use crate::ffi::create_component;

pub static mut PLATFORM_CAPABILITIES: AtomicI32 = AtomicI32::new(-1);

struct AddInDescription {
    name: &'static [u16],
    connection: Arc<Option<&'static Connection>>,

    some_prop_container: i32,
}

impl AddInDescription {
    pub fn new() -> Self {
        Self {
            name: &utf16_null!("MyAddIn"),
            connection: Arc::new(None),
            some_prop_container: 0,
        }
    }

    fn iterate(&mut self) -> Result<(), &str> {
        if self.some_prop_container >= 105 {
            return Err("container is too big");
        }
        self.some_prop_container += 1;
        Ok(())
    }
}

impl AddIn for AddInDescription {
    fn init(&mut self, interface: &'static Connection) -> bool {
        self.connection = Arc::new(Some(interface));
        self.some_prop_container = 100;
        true
    }

    fn add_in_name(&self) -> &'static [u16] {
        self.name
    }

    fn call_function(&mut self, name: &str, params: &[ParamValue]) -> Option<ParamValue> {
        match name.trim_end() {
            "iterate" => {
                let Err(_e) = self.iterate() else { return None};
                None
            }
            "err" => {
                let Some(con) = self.connection.as_deref() else { return None };
                con.add_error(13, "err", "error that was bound to happen");
                None
            }
            "timer" => {
                let ParamValue::I32(duration_ms) = params[0] else { return None };
                let con_clone = self.connection.clone();
                thread::spawn(move || {
                    let Some(con) = con_clone.as_deref() else { return };
                    if duration_ms <= 0 {
                        return;
                    }
                    thread::sleep(Duration::from_millis(duration_ms as u64));
                    con.external_event("timer", "timer", "timer");
                });

                Some(ParamValue::I32(duration_ms))
            }
            _ => None,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<ParamValue> {
        match name {
            "prop" => Some(ParamValue::I32(self.some_prop_container)),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: &ParamValue) -> bool {
        match name {
            "prop" => {
                let ParamValue::I32(val) = value else { return false };
                self.some_prop_container = *val;
                true
            }
            _ => false,
        }
    }

    fn list_functions(&self) -> Vec<ComponentFuncDescription> {
        vec![
            ComponentFuncDescription::new::<0>(&utf16_null!("iterate"), false, &[]),
            ComponentFuncDescription::new::<0>(&utf16_null!("err"), false, &[]),
            ComponentFuncDescription::new::<1>(
                &utf16_null!("timer"),
                true,
                &[Some(ParamValue::I32(0))],
            ),
        ]
    }

    fn list_parameters(&self) -> Vec<ComponentPropDescription> {
        vec![ComponentPropDescription {
            name: &utf16_null!("prop"),
            readable: true,
            writable: true,
        }]
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn GetClassObject(name: *const u16, component: *mut *mut c_void) -> c_long {
    match *name as u8 {
        b'1' => {
            let my_add_in_container = AddInContainer::new(AddInDescription::new());
            create_component(component, my_add_in_container)
        }
        _ => 0,
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn DestroyObject(component: *mut *mut c_void) -> c_long {
    destroy_component(component)
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetClassNames() -> *const u16 {
    // small strings for performance
    utf16_null!("1").as_ptr()
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn SetPlatformCapabilities(capabilities: c_int) -> c_int {
    PLATFORM_CAPABILITIES.store(capabilities, Ordering::Relaxed);
    3
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetAttachType() -> AttachType {
    AttachType::CanAttachAny
}
