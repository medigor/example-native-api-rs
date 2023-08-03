use std::{
    ffi::{c_ulong, c_void},
    ptr::{self, NonNull},
};

#[repr(C)]
struct MemoryManagerVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    alloc_memory:
        unsafe extern "system" fn(&MemoryManager, *mut *mut c_void, c_ulong) -> bool,
    free_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void),
}

#[repr(C)]
pub struct MemoryManager {
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
}
