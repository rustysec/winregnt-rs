#![cfg(target_os = "windows")]

mod api;
mod error;
mod reg_key_iterator;
mod reg_value_iterator;
mod unicode_string;

pub use crate::api::*;
use crate::error::Error;
use crate::reg_key_iterator::*;
use crate::reg_value_iterator::*;
use crate::unicode_string::*;
use std::ffi::OsString;
use std::mem::zeroed;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::shared::ntdef::{
    InitializeObjectAttributes, HANDLE, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE,
};
use winapi::um::winnt::KEY_ALL_ACCESS;

pub type Result<T> = std::result::Result<T, error::Error>;

/// Entry point for all registry access
pub struct RegKey {
    handle: HANDLE,
    name: Vec<u16>,
    u: UnicodeString,
}

impl Drop for RegKey {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                NtClose(self.handle);
            }
        }
    }
}

impl RegKey {
    /// opens a registry key
    ///
    /// # Examples
    /// ```
    /// let reg = RegKey::open(r"\Registry\User").unwrap();
    /// ```
    ///
    pub fn open<S: Into<String> + Clone>(name: S) -> Result<RegKey> {
        let name = name.into();
        let mut key = RegKey {
            handle: unsafe { zeroed() },
            name: {
                let mut t = OsString::from(&name).encode_wide().collect::<Vec<u16>>();
                t.push(0x00);
                t
            },
            u: unsafe { zeroed() },
        };
        key.u = UnicodeString::from(&key.name);

        let mut object_attr: OBJECT_ATTRIBUTES = unsafe { zeroed() };
        unsafe {
            InitializeObjectAttributes(
                &mut object_attr,
                &mut key.u.0,
                OBJ_CASE_INSENSITIVE,
                null_mut(),
                null_mut(),
            );
        }
        match unsafe { NtOpenKey(&mut key.handle, KEY_ALL_ACCESS, &object_attr) } {
            0 => Ok(key),
            err => Err(Error::KeyError(name, err)),
        }
    }

    /// get an sub key enumerator
    pub fn enum_keys(&self) -> RegKeyIterator {
        RegKeyIterator::new(&self)
    }

    /// get a key value iterator
    pub fn enum_values(&self) -> RegValueIterator {
        RegValueIterator::new(&self.handle)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn open() {
        use crate::RegKey;
        match RegKey::open(
            r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion\Run".to_owned(),
        ) {
            Ok(_) => {
                assert!(true);
            }
            _ => assert!(false),
        }
    }
}
