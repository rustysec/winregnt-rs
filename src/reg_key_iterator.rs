use crate::api::*;
use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use winapi::shared::minwindef::ULONG;
use winapi::shared::ntdef::HANDLE;

pub struct RegKeyIterator<'a> {
    handle: &'a HANDLE,
    index: ULONG,
}

impl<'a> RegKeyIterator<'a> {
    pub fn new(handle: &'a HANDLE) -> RegKeyIterator<'a> {
        RegKeyIterator {
            handle: handle,
            index: 0,
        }
    }
}

impl<'a> Iterator for RegKeyIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut result_length: ULONG = 0;
        unsafe {
            NtEnumerateKey(
                *self.handle,
                self.index,
                KeyInformationClass::KeyBasicInformation,
                null_mut() as _,
                0,
                &mut result_length,
            )
        };

        let mut data: Vec<u8> = vec![0; result_length as _];
        match unsafe {
            NtEnumerateKey(
                *self.handle,
                self.index,
                KeyInformationClass::KeyBasicInformation,
                data.as_mut_ptr() as *mut _,
                data.len() as _,
                &mut result_length,
            )
        } {
            0 => {
                let value: KeyBasicInformation =
                    unsafe { std::ptr::read(data.as_ptr() as *const _) };
                let name: &[u16] = unsafe {
                    std::slice::from_raw_parts(
                        data[size_of::<KeyBasicInformation>()..].as_ptr() as _,
                        (value.name_length / 2) as _,
                    )
                };
                match OsString::from_wide(&name).into_string() {
                    Ok(s) => {
                        self.index += 1;
                        Some(s)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn enumerate() {
        use crate::RegKey;
        let key =
            RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion".to_owned())
                .unwrap();
        let mut iter = key.enum_keys();
        assert!(iter.next().is_some());
    }
}
