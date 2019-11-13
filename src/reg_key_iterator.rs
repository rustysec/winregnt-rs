use crate::api::*;
use crate::RegKey;
use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::ULONG;
use winapi::shared::ntdef::HANDLE;

/// iterator over registry keys
pub struct RegKeyIterator<'a> {
    handle: &'a HANDLE,
    index: ULONG,
    name: &'a [u16],
}

impl<'a> RegKeyIterator<'a> {
    /// get an iterator for a `RegKey`
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
            Some(data) => match data.len() >= size_of::<KeyBasicInformation>() {
                true => {
                    let value: KeyBasicInformation =
                        unsafe { std::ptr::read(data.as_ptr() as *const _) };
                    let name: Vec<u16> = {
                        let length = (value.name_length / 2) as usize;
                        let data = data
                            .iter()
                            .copied()
                            .skip(size_of::<KeyBasicInformation>())
                            .collect::<Vec<u8>>();

                        match data.len() >= length {
                            true => unsafe {
                                std::slice::from_raw_parts::<u16>(data.as_ptr() as _, length)
                            }
                            .to_vec(),
                            false => Vec::new(),
                        }
                    };

                    match OsString::from_wide(&name).into_string() {
                        Ok(s) => {
                            self.index += 1;
                            Some(RegSubkey {
                                name: s,
                                parent: self.name,
                            })
                        }
                        Err(_) => None,
                    }
                }
                false => None,
            },
            _ => None,
        }
    }
}

/// child key
pub struct RegSubkey<'a> {
    name: String,
    parent: &'a [u16],
}

impl<'a> RegSubkey<'a> {
    /// returns a `RegKey`
    pub fn open(&'a self) -> Result<RegKey, ()> {
        let parent = {
            let mut p = self.parent.to_vec();
            p.pop();
            p
        };

        let mut s = OsString::from_wide(&parent).into_string().map_err(|_| ())?;
        s.push_str("\\");
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
