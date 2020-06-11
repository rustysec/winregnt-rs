//! A Rust interface to `Nt*` series of windows registry APIs. [winreg](https://github.com/gentoo90/winreg-rs) is a
//! fantastic library but uses the common (and friendly) win32 APIs to interact with the registry. This leaves
//! some blind spots when dealing with `null` characters which are permitted by the `Nt` functions and _not_ by
//! Win32. Some information about this can be found
//! [here](https://docs.microsoft.com/en-us/sysinternals/downloads/reghide).
//!
//!
//! ## Usage
//! In your `cargo.toml`:
//!
//! ```toml
//! winregnt = { git = "https://github.com/rustysec/winregnt-rs" }
//! ```
//!
//! `main.rs`:
//!
//! ```no_run
//! use winregnt::RegKey;
//!
//! fn main() {
//!     let key =
//!         RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
//!     key.enum_keys().for_each(|k| println!("- {}", k));
//! }
//! ```
//!

#![cfg(target_os = "windows")]
#![warn(missing_docs)]

mod api;
mod error;
mod reg_key_iterator;
mod reg_value_iterator;
mod unicode_string;

pub use crate::api::*;
pub use crate::error::*;
use crate::reg_key_iterator::*;
use crate::reg_value_iterator::*;
use crate::unicode_string::*;
use std::{ffi::OsString, mem::zeroed, os::windows::ffi::OsStrExt, ptr::null_mut};
use winapi::{
    shared::{
        ntdef::{InitializeObjectAttributes, HANDLE, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE},
        ntstatus::{
            STATUS_ACCESS_DENIED, STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_HANDLE,
            STATUS_OBJECT_NAME_NOT_FOUND,
        },
    },
    um::winnt::{KEY_READ, KEY_WRITE},
};

/// Result wrapping WinRegNt errors
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
    /// opens a registry key as read only
    ///
    /// # Examples
    ///
    /// ```
    /// use winregnt::RegKey;
    /// assert!(RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion\Run").is_ok());
    /// ```
    ///
    pub fn open<S: AsRef<str>>(name: S) -> Result<RegKey> {
        Self::open_key(name, KEY_READ)
    }

    /// opens a registry key with write permissions
    ///
    /// # Examples
    ///
    /// ```
    /// use winregnt::RegKey;
    /// assert!(RegKey::open_write(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion\Run").is_ok());
    /// ```
    ///
    pub fn open_write<S: AsRef<str>>(name: S) -> Result<RegKey> {
        Self::open_key(name, KEY_WRITE)
    }

    /// get an sub key enumerator
    pub fn enum_keys(&self) -> RegKeyIterator {
        RegKeyIterator::new(&self)
    }

    /// get a key value iterator
    pub fn enum_values(&self) -> RegValueIterator {
        RegValueIterator::new(&self.handle)
    }

    /// delete the current key
    pub fn delete(&self) -> Result<()> {
        match unsafe { api::NtDeleteKey(self.handle) } as i32 {
            STATUS_ACCESS_DENIED => Err(RegKeyError::DeleteAccessDenied.into()),
            STATUS_INVALID_HANDLE => Err(RegKeyError::DeleteInvalidHandle.into()),
            _ => Ok(()),
        }
    }

    /// delete a value
    pub fn delete_value<S: AsRef<str>>(&self, value_name: S) -> Result<()> {
        let mut unicode_string = UnicodeString::from(value_name.as_ref());
        match unsafe { NtDeleteValueKey(self.handle, &mut unicode_string.0) } as i32 {
            STATUS_ACCESS_DENIED => Err(crate::error::RegValueError::AccessDenied.into()),
            STATUS_INSUFFICIENT_RESOURCES => {
                Err(crate::error::RegValueError::InsufficientResources.into())
            }
            STATUS_INVALID_HANDLE => Err(RegValueError::InvalidHandle.into()),
            STATUS_OBJECT_NAME_NOT_FOUND => Err(RegValueError::NameNotFound.into()),
            _ => Ok(()),
        }
    }

    fn open_key<S: AsRef<str>>(name: S, permission: u32) -> Result<RegKey> {
        let mut key = RegKey {
            handle: unsafe { zeroed() },
            name: {
                let mut t = OsString::from(name.as_ref())
                    .encode_wide()
                    .collect::<Vec<u16>>();
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

        match unsafe { NtOpenKey(&mut key.handle, permission, &object_attr) } {
            0 => Ok(key),
            err => Err(Error::KeyError(name.as_ref().to_string(), err)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn open() {
        use crate::RegKey;
        assert!(
            RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion\Run",)
                .is_ok()
        );
    }
}
