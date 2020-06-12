use crate::api::RtlInitUnicodeString;
use std::ffi::OsString;
use std::mem::zeroed;
use std::os::windows::ffi::OsStrExt;
use winapi::shared::ntdef::UNICODE_STRING;

pub(crate) struct UnicodeString(pub UNICODE_STRING, Vec<u16>);

impl Default for UnicodeString {
    fn default() -> Self {
        UnicodeString(unsafe { std::mem::zeroed() }, Vec::new())
    }
}

impl From<&str> for UnicodeString {
    fn from(input: &str) -> Self {
        let mut u: UNICODE_STRING = unsafe { zeroed() };
        let mut o = OsString::from(input).encode_wide().collect::<Vec<u16>>();
        o.push(0x00);
        o.push(0x00);

        unsafe {
            RtlInitUnicodeString(&mut u, o.as_ptr());
        }
        UnicodeString(u, o)
    }
}

impl From<&Vec<u16>> for UnicodeString {
    fn from(input: &Vec<u16>) -> Self {
        let mut u: UNICODE_STRING = unsafe { zeroed() };
        unsafe {
            RtlInitUnicodeString(&mut u, input.as_ptr());
        }
        UnicodeString(u, input.to_vec())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unicode() {
        use crate::UnicodeString;
        let s = UnicodeString::from("testing");
        assert_eq!(s.0.Length, 14);
    }
}
