/// Null terminated utf-16 static string, used for names
#[macro_export]
macro_rules! name {
    ($text:expr) => {
        &utf16_lit::utf16_null!($text)
    };
}
