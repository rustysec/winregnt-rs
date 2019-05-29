use crate::api::*;
use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::ULONG;
use winapi::shared::ntdef::HANDLE;

pub struct RegValueIterator<'a> {
    handle: &'a HANDLE,
    index: ULONG,
}

impl<'a> RegValueIterator<'a> {
    pub fn new(handle: &'a HANDLE) -> RegValueIterator<'a> {
        RegValueIterator {
            handle: handle,
            index: 0,
        }
    }
}

impl<'a> Iterator for RegValueIterator<'a> {
    type Item = RegValueItem;

    fn next(&mut self) -> Option<RegValueItem> {
        match enumerate_value_key(*self.handle, self.index) {
            Some(data) => {
                self.index += 1;
                Some(RegValueItem::from(data))
            }
            _ => None,
        }
    }
}

pub struct RegValueItem {
    name: Vec<u16>,
    value: RegValue,
}

impl RegValueItem {
    pub fn name(&self) -> String {
        OsString::from_wide(&self.name).into_string().unwrap()
    }

    pub fn value(&self) -> RegValue {
        self.value.clone()
    }
}

impl std::fmt::Display for RegValueItem {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.name())
    }
}

impl From<Vec<u8>> for RegValueItem {
    fn from(data: Vec<u8>) -> RegValueItem {
        let start = size_of::<KeyValueFullInformation>();
        let value: KeyValueFullInformation = unsafe { std::ptr::read(data.as_ptr() as *const _) };
        let name = unsafe {
            std::slice::from_raw_parts::<u16>(
                data[start..].as_ptr() as _,
                (value.name_length / 2) as usize,
            )
        };
        RegValueItem {
            name: name.to_vec(),
            value: RegValue::new(&value, &data),
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
        let mut iter = key.enum_values();
        assert!(iter.next().is_some());
    }
}
