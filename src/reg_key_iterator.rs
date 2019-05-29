use crate::api::*;
use crate::RegKey;
use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::ULONG;
use winapi::shared::ntdef::HANDLE;

pub struct RegKeyIterator<'a> {
    handle: &'a HANDLE,
    index: ULONG,
    name: &'a [u16],
}

impl<'a> RegKeyIterator<'a> {
    pub fn new(key: &'a RegKey) -> RegKeyIterator<'a> {
        RegKeyIterator {
            handle: &key.handle,
            index: 0,
            name: &key.name,
        }
    }
}

impl<'a> Iterator for RegKeyIterator<'a> {
    type Item = RegSubkey<'a>;

    fn next(&mut self) -> Option<RegSubkey<'a>> {
        match enumerate_key(*self.handle, self.index) {
            Some(data) => {
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
                        Some(RegSubkey {
                            name: s,
                            parent: self.name,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub struct RegSubkey<'a> {
    name: String,
    parent: &'a [u16],
}

impl<'a> RegSubkey<'a> {
    pub fn open(&'a self) -> Result<RegKey, ()> {
        let mut s = OsString::from_wide(self.parent).into_string().unwrap();
        s.push_str(&self.name);
        RegKey::open(s)
    }
}

impl<'a> ::std::fmt::Display for RegSubkey<'a> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.name)
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
