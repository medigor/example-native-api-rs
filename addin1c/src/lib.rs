mod ffi;
mod macros;
mod simple;

pub use ffi::{
    create_component, destroy_component, Addin as RawAddin, AttachType, Connection, ParamValue, Tm,
    Variant,
};
pub use simple::{Addin as SimpleAddin, MethodInfo, Methods, PropInfo};
pub use utf16_lit::utf16_null;
