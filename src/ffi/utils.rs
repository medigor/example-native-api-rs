use std::slice::from_raw_parts;

pub unsafe fn get_str<'a>(s: *const u16) -> &'a [u16] {
    unsafe fn strlen(s: *const u16) -> usize {
        let mut i = 0;
        while *s.add(i) != 0 {
            i += 1;
        }
        i + 1
    }

    let len = strlen(s);
    from_raw_parts(s, len)
}

#[cfg(target_family = "unix")]
pub fn os_string(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[cfg(target_family = "windows")]
pub fn os_string(s: &str) -> Vec<u16> {
    let os_str = std::ffi::OsStr::new(s);
    std::os::windows::prelude::OsStrExt::encode_wide(os_str)
        .chain(Some(0).into_iter())
        .collect()
}

pub fn from_os_string(s: &[u16]) -> String {
    String::from_utf16_lossy(s)
        .trim_end_matches(char::from(0))
        .to_string()
}
