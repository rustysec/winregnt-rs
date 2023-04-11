use crate::{
    api::*,
    error::{self, Error},
    RegKey, Result,
};
use std::{
    ffi::OsString,
    mem::size_of,
    os::windows::ffi::OsStringExt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use winapi::shared::{minwindef::ULONG, ntdef::HANDLE};

/// iterator over registry keys
pub struct RegKeyIterator {
    handle: Arc<AtomicUsize>,
    index: ULONG,
    name: Vec<u16>,
}

impl RegKeyIterator {
    /// get an iterator for a `RegKey`
    pub fn new(key: &RegKey) -> RegKeyIterator {
        RegKeyIterator {
            handle: key.handle.clone(),
            index: 0,
            name: key.name.clone(),
        }
    }

    fn handle(&self) -> HANDLE {
        self.handle.load(Ordering::SeqCst) as HANDLE
    }
}

impl Iterator for RegKeyIterator {
    type Item = RegSubkey;

    fn next(&mut self) -> Option<RegSubkey> {
        match enumerate_key(self.handle(), self.index) {
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
                                        parent: self.name.clone(),
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
pub struct RegSubkey {
    name: String,
    parent: Vec<u16>,
}

impl RegSubkey {
    /// returns a `RegKey`
    pub fn open(&self) -> Result<RegKey> {
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
    pub fn open_write(&self) -> Result<RegKey> {
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

    /// returns the name of the subkey
    pub fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl ::std::fmt::Display for RegSubkey {
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
