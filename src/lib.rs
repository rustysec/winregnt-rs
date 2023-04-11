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

pub use crate::{api::*, error::*, reg_key_iterator::RegSubkey, reg_value_iterator::RegValueItem};
use crate::{reg_key_iterator::*, reg_value_iterator::*, unicode_string::*};
use std::{
    ffi::OsString,
    mem::zeroed,
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use winapi::{
    shared::{
        ntdef::{InitializeObjectAttributes, HANDLE, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE},
        ntstatus::{
            STATUS_ACCESS_DENIED, STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_HANDLE,
            STATUS_OBJECT_NAME_NOT_FOUND,
        },
    },
    um::winnt::{
        DELETE, KEY_READ, KEY_SET_VALUE, KEY_WRITE, REG_BINARY, REG_DWORD, REG_NONE, REG_QWORD,
        REG_SZ,
    },
};

/// Result wrapping WinRegNt errors
pub type Result<T> = std::result::Result<T, error::Error>;

/// Entry point for all registry access
#[derive(Clone)]
pub struct RegKey {
    handle: Arc<AtomicUsize>,
    name: Vec<u16>,
    u: UnicodeString,
}

impl Drop for RegKey {
    fn drop(&mut self) {
        let handle = self.handle();
        if !handle.is_null() {
            unsafe {
                NtClose(handle);
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
        Self::open_key(name, KEY_WRITE | DELETE | KEY_SET_VALUE)
    }

    /// get an sub key enumerator
    pub fn enum_keys(&self) -> RegKeyIterator {
        RegKeyIterator::new(&self)
    }

    /// get a key value iterator
    pub fn enum_values(&self) -> RegValueIterator {
        RegValueIterator::new(self.handle.clone())
    }

    /// delete the current key
    pub fn delete(&self) -> Result<()> {
        match unsafe { api::NtDeleteKey(self.handle.load(Ordering::SeqCst) as HANDLE) } as i32 {
            STATUS_ACCESS_DENIED => Err(RegKeyError::DeleteAccessDenied.into()),
            STATUS_INVALID_HANDLE => Err(RegKeyError::DeleteInvalidHandle.into()),
            _ => Ok(()),
        }
    }

    /// delete a value
    pub fn delete_value<S: AsRef<str>>(&self, value_name: S) -> Result<()> {
        let unicode_string = UnicodeString::from(value_name.as_ref());
        match unsafe { NtDeleteValueKey(self.handle(), &unicode_string.0 as *const _ as *mut _) }
            as i32
        {
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
            handle: Arc::new(Default::default()),
            name: {
                let mut t = OsString::from(name.as_ref())
                    .encode_wide()
                    .collect::<Vec<u16>>();
                t.push(0x00);
                t
            },
            u: Default::default(),
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

        match unsafe {
            let mut handle: HANDLE = zeroed();

            let temp = NtOpenKey(&mut handle, permission, &object_attr);

            key.handle.store(handle as _, Ordering::SeqCst);

            temp
        } {
            0 => Ok(key),
            err => Err(Error::KeyError(name.as_ref().to_string(), err)),
        }
    }

    /// Create or update a binary value `name` with `value`
    pub fn write_binary_value<S: AsRef<str>, V: AsRef<[u8]>>(
        &mut self,
        name: S,
        value: V,
    ) -> Result<()> {
        let unicode_name = UnicodeString::from(name.as_ref());
        match unsafe {
            NtSetValueKey(
                self.handle(),
                &unicode_name.0 as *const _ as *mut _,
                0,
                REG_BINARY,
                value.as_ref() as *const _ as *mut _,
                value.as_ref().len() as _,
            )
        } {
            0 => Ok(()),
            err => Err(RegValueError::Write(err).into()),
        }
    }

    /// Create or update a binary value `name` with `value`
    pub fn write_string_value<S: AsRef<str>, V: AsRef<str>>(
        &mut self,
        name: S,
        value: V,
    ) -> Result<()> {
        let unicode_name = UnicodeString::from(name.as_ref());

        let mut o = OsString::from(value.as_ref())
            .encode_wide()
            .collect::<Vec<u16>>();
        o.push(0x00);

        match unsafe {
            NtSetValueKey(
                self.handle(),
                &unicode_name.0 as *const _ as *mut _,
                0,
                REG_SZ,
                o.as_mut_ptr() as _,
                (o.len() * 2) as _,
            )
        } {
            0 => Ok(()),
            err => Err(RegValueError::Write(err).into()),
        }
    }

    /// Create or update a binary value `name` with `value`
    pub fn write_dword_value<S: AsRef<str>>(&mut self, name: S, value: u32) -> Result<()> {
        let unicode_name = UnicodeString::from(name.as_ref());
        match unsafe {
            NtSetValueKey(
                self.handle(),
                &unicode_name.0 as *const _ as *mut _,
                0,
                REG_DWORD,
                &mut value.clone() as *const _ as *mut _,
                std::mem::size_of::<u32>() as _,
            )
        } {
            0 => Ok(()),
            err => Err(RegValueError::Write(err).into()),
        }
    }

    /// Create or update a `NONE` value `name` with `value`
    pub fn write_qword_value<S: AsRef<str>>(&mut self, name: S, value: u64) -> Result<()> {
        let unicode_name = UnicodeString::from(name.as_ref());
        match unsafe {
            NtSetValueKey(
                self.handle(),
                &unicode_name.0 as *const _ as *mut _,
                0,
                REG_QWORD,
                &mut value.clone() as *const _ as *mut _,
                std::mem::size_of::<u64>() as _,
            )
        } {
            0 => Ok(()),
            err => Err(RegValueError::Write(err).into()),
        }
    }

    /// Create or update a `NONE` value `name` with `value`
    pub fn write_none_value<S: AsRef<str>, V: AsRef<[u8]>>(
        &mut self,
        name: S,
        value: V,
    ) -> Result<()> {
        let unicode_name = UnicodeString::from(name.as_ref());
        match unsafe {
            NtSetValueKey(
                self.handle(),
                &unicode_name.0 as *const _ as *mut _,
                0,
                REG_NONE,
                value.as_ref() as *const _ as *mut _,
                value.as_ref().len() as _,
            )
        } {
            0 => Ok(()),
            err => Err(RegValueError::Write(err).into()),
        }
    }

    fn handle(&self) -> HANDLE {
        self.handle.load(Ordering::SeqCst) as HANDLE
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
