use crate::{
    api::*,
    error::{self, Error},
    RegKey, Result,
};
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
            Some(data) => {
                if data.len() >= size_of::<KeyBasicInformation>() {
                    match KeyBasicInformation::new(&data) {
                        Ok(value) => {
                            let name: Vec<u16> = {
                                let length = (value.name_length / 2) as usize;

                                let data = data
                                    .iter()
                                    .copied()
                                    .skip(size_of::<KeyBasicInformation>())
                                    .take(value.name_length as _)
                                    .collect::<Vec<u8>>();

                                if data.len() >= length {
                                    data.chunks_exact(2)
                                        .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                                        .take(length)
                                        .collect::<Vec<u16>>()
                                } else {
                                    Vec::new()
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
                        _ => None,
                    }
                } else {
                    None
                }
            }
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
    pub fn open(&'a self) -> Result<RegKey> {
        let parent = {
            let mut p = self.parent.to_vec();
            p.pop();
            p
        };

        let mut s = OsString::from_wide(&parent)
            .into_string()
            .map_err(|_| Into::<Error>::into(error::SubKeyError::ConvertName))?;
        s.push_str("\\");
        s.push_str(&self.name);
        RegKey::open(s)
    }

    /// returns a `RegKey`
    pub fn open_write(&'a self) -> Result<RegKey> {
        let parent = {
            let mut p = self.parent.to_vec();
            p.pop();
            p
        };

        let mut s = OsString::from_wide(&parent)
            .into_string()
            .map_err(|_| Into::<Error>::into(error::SubKeyError::ConvertName))?;
        s.push_str("\\");
        s.push_str(&self.name);
        RegKey::open_write(s)
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
