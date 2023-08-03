pub mod add_in;
mod ffi;

use crate::ffi::create_component;
use add_in::{
    AddIn, AddInContainer, ComponentFuncDescription, ComponentPropDescription,
};
use ffi::{
    connection::Connection,
    destroy_component,
    types::ParamValue,
    utils::{from_os_string, os_string},
    AttachType,
};

use color_eyre::eyre::{eyre, Result};
use std::{
    ffi::{c_int, c_long, c_void},
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use utf16_lit::utf16_null;

pub static mut PLATFORM_CAPABILITIES: AtomicI32 = AtomicI32::new(-1);

struct AddInDescription {
    name: &'static [u16],
    connection: Arc<Option<&'static Connection>>,

    functions: Vec<(
        ComponentFuncDescription,
        fn(&mut Self, &[ParamValue]) -> Result<Option<ParamValue>>,
    )>,

    some_prop_container: i32,
}

impl AddInDescription {
    pub fn new() -> Self {
        Self {
            name: &utf16_null!("MyAddIn"),
            connection: Arc::new(None),

            functions: vec![
                (
                    ComponentFuncDescription::new::<0>(
                        vec![
                            &utf16_null!("Итерировать"),
                            &utf16_null!("Iterate"),
                        ],
                        false,
                        &[],
                    ),
                    Self::iterate,
                ),
                (
                    ComponentFuncDescription::new::<1>(
                        vec![&utf16_null!("Таймер"), &utf16_null!("Timer")],
                        true,
                        &[Some(ParamValue::I32(1000))],
                    ),
                    Self::timer,
                ),
                (
                    ComponentFuncDescription::new::<0>(
                        vec![
                            &utf16_null!("ПолучитьХэТэТэПэ"),
                            &utf16_null!("FetchHTTP"),
                        ],
                        true,
                        &[],
                    ),
                    Self::fetch,
                ),
            ],

            some_prop_container: 0,
        }
    }

    fn iterate(
        &mut self,
        _params: &[ParamValue],
    ) -> Result<Option<ParamValue>> {
        if self.some_prop_container >= 105 {
            return Err(eyre!("Prop is too big"));
        }
        self.some_prop_container += 1;
        Ok(None)
    }

    fn timer(&mut self, params: &[ParamValue]) -> Result<Option<ParamValue>> {
        let sleep_duration_ms = match params.get(0) {
            Some(ParamValue::I32(val)) => *val,
            _ => return Err(eyre!("Invalid parameter")),
        };

        let connection = self.connection.clone();
        let name = from_os_string(self.name);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            if let Some(connection) = &*connection {
                connection.external_event(&name, "TimerEnd", "OK");
            }
        });

        Ok(Some(ParamValue::I32(sleep_duration_ms)))
    }

    fn fetch(&mut self, _params: &[ParamValue]) -> Result<Option<ParamValue>> {
        let Ok(result) = ureq::post("https://echo.hoppscotch.io").send_string("smth") else {return Err(eyre!("Failed to fetch"));};
        let Ok(body) = result.into_string() else { return Err(eyre!("Failed to get body"));};
        Ok(Some(ParamValue::Str(os_string(&body))))
    }
}

impl AddIn for AddInDescription {
    fn init(&mut self, interface: &'static Connection) -> bool {
        interface.set_event_buffer_depth(10);
        self.connection = Arc::new(Some(interface));
        self.some_prop_container = 100;
        true
    }

    fn add_in_name(&self) -> &'static [u16] {
        self.name
    }

    fn call_function(
        &mut self,
        name: &str,
        params: &[ParamValue],
    ) -> Result<Option<ParamValue>> {
        let func = self
            .functions
            .iter()
            .find(|(desc, _)| desc.str_names().iter().any(|n| n == name));

        let Some(func) = func.map(|(_, callback)| callback) else { return Err(eyre!("No function with such name")) };
        func(self, params)
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

    fn list_functions(&self) -> Vec<&ComponentFuncDescription> {
        self.functions.iter().map(|(desc, _)| desc).collect()
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
pub unsafe extern "C" fn GetClassObject(
    name: *const u16,
    component: *mut *mut c_void,
) -> c_long {
    match *name as u8 {
        b'1' => {
            let my_add_in_container =
                AddInContainer::new(AddInDescription::new());
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
    AttachType::Any
}
