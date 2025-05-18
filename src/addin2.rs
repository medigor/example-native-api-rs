use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use addin1c::{
    cstr1c, name, AddinResult, CStr1C, CString1C, Connection, MethodInfo, Methods, PropInfo,
    SimpleAddin, Variant,
};

pub struct Addin2 {
    last_error: Option<Box<dyn Error>>,
    prop1: i32,
    connection: Option<&'static Connection>,
    timer_enabled: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Addin2 {
    pub fn new() -> Addin2 {
        Addin2 {
            last_error: None,
            prop1: 0,
            connection: None,
            timer_enabled: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    fn last_error(&mut self, value: &mut Variant) -> AddinResult {
        match &self.last_error {
            Some(err) => value.set_str1c(err.to_string()).map_err(|e| e.into()),
            None => value.set_str1c("").map_err(|e| e.into()),
        }
    }

    fn method1(&mut self, param: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let value = param.get_i32()?;
        self.prop1 = value;
        ret_value.set_i32(value * 2);
        Ok(())
    }

    fn method2(
        &mut self,
        param1: &mut Variant,
        param2: &mut Variant,
        ret_value: &mut Variant,
    ) -> AddinResult {
        let value1 = param1.get_i32()?;
        let value2 = param2.get_i32()?;
        self.prop1 = value1 + value2;
        ret_value.set_i32(self.prop1);
        Ok(())
    }

    fn start_timer(&mut self, duration: &mut Variant, _ret_value: &mut Variant) -> AddinResult {
        let Some(connection) = self.connection else {
            return Err("Нет интерфейса".into());
        };
        let duration = duration.get_i32()?;
        let duration = duration.try_into()?;

        if let Some(handle) = self.thread_handle.as_ref() {
            if !handle.is_finished() {
                return Err("Timer is started".into());
            }
        }

        self.timer_enabled.store(true, Ordering::Relaxed);
        let enabled = self.timer_enabled.clone();
        connection.set_event_buffer_depth(100);
        let handle = thread::spawn(move || {
            let mut counter = 0;
            loop {
                thread::sleep(Duration::from_millis(duration));
                if enabled.load(Ordering::Relaxed) {
                    counter += 1;
                    connection.external_event(
                        cstr1c!("Class2"),
                        cstr1c!("Timer"),
                        CString1C::new(&format!("{counter}")),
                    );
                } else {
                    break;
                }
            }
            connection.external_event(cstr1c!("Class2"), cstr1c!("TimerShutdown"), cstr1c!(""));
        });
        self.thread_handle = Some(handle);
        Ok(())
    }

    fn stop_timer(&mut self, _ret_value: &mut Variant) -> AddinResult {
        self.timer_enabled.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn set_prop1(&mut self, value: &Variant) -> AddinResult {
        self.prop1 = value.get_i32()?;
        Ok(())
    }

    fn get_prop1(&mut self, value: &mut Variant) -> AddinResult {
        value.set_i32(self.prop1);
        Ok(())
    }
}

impl Drop for Addin2 {
    fn drop(&mut self) {
        self.timer_enabled.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl SimpleAddin for Addin2 {
    fn name() -> &'static CStr1C {
        name!("Class2")
    }

    fn init(&mut self, interface: &'static Connection) -> bool {
        self.connection = Some(interface);
        true
    }

    fn save_error(&mut self, err: Option<Box<dyn Error>>) {
        self.last_error = err;
    }

    fn methods() -> &'static [MethodInfo<Self>] {
        &[
            MethodInfo {
                name: name!("Method1"),
                method: Methods::Method1(Self::method1),
            },
            MethodInfo {
                name: name!("Method2"),
                method: Methods::Method2(Self::method2),
            },
            MethodInfo {
                name: name!("StartTimer"),
                method: Methods::Method1(Self::start_timer),
            },
            MethodInfo {
                name: name!("StopTimer"),
                method: Methods::Method0(Self::stop_timer),
            },
        ]
    }

    fn properties() -> &'static [PropInfo<Self>] {
        &[
            PropInfo {
                name: name!("Prop1"),
                getter: Some(Self::get_prop1),
                setter: Some(Self::set_prop1),
            },
            PropInfo {
                name: name!("LastError"),
                getter: Some(Self::last_error),
                setter: None,
            },
        ]
    }
}
