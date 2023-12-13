/// Null terminated utf-16 static string, used for names
#[macro_export]
macro_rules! name {
    ($text:expr) => {
        &addin1c::utf16_null!($text)
    };
}
