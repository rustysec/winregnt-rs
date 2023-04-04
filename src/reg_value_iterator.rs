use crate::{api::*, error, Result};
use std::{
    cell::RefCell, convert::TryFrom, ffi::OsString, mem::size_of, os::windows::ffi::OsStringExt,
    rc::Rc,
};
use winapi::shared::{minwindef::ULONG, ntdef::HANDLE};

/// get an iterator of key values
pub struct RegValueIterator {
    handle: Rc<RefCell<HANDLE>>,
    index: ULONG,
}

impl RegValueIterator {
    /// create a new RegValueIterator from a handle
    pub fn new(handle: Rc<RefCell<HANDLE>>) -> RegValueIterator {
        RegValueIterator { handle, index: 0 }
    }
}

impl Iterator for RegValueIterator {
    type Item = RegValueItem;

    fn next(&mut self) -> Option<RegValueItem> {
        match enumerate_value_key(*self.handle.borrow(), self.index) {
            Some(data) => {
                self.index += 1;
                RegValueItem::try_from(data).ok()
            }
            _ => None,
        }
    }
}

/// defines a registry value (name and data)
pub struct RegValueItem {
    name: Vec<u16>,
    value: RegValue,
}

impl RegValueItem {
    /// returns the name of the value
    pub fn name(&self) -> Result<String> {
        OsString::from_wide(&self.name)
            .into_string()
            .map_err(|_| error::RegValueError::ConvertName.into())
    }

    /// returns the `RegValue`
    pub fn value(&self) -> RegValue {
        self.value.clone()
    }
}

impl std::fmt::Display for RegValueItem {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.name().unwrap_or_default())
    }
}

impl TryFrom<Vec<u8>> for RegValueItem {
    type Error = error::Error;

    fn try_from(data: Vec<u8>) -> Result<Self> {
        let start = size_of::<KeyValueFullInformation>();
        if data.len() >= start {
            let value = KeyValueFullInformation::new(&data).map_err(Into::<Self::Error>::into)?;

            let length = (value.name_length / 2) as usize;

            let name_data = data.iter().copied().skip(start).collect::<Vec<u8>>();
            if name_data.len() >= length {
                let name = name_data
                    .chunks_exact(2)
                    .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                    .take(length)
                    .collect::<Vec<u16>>();

                Ok(RegValueItem {
                    name,
                    value: RegValue::new(&value, &data).map_err(|e| {
                        Into::<error::Error>::into(error::RegValueError::ValueData(e.to_string()))
                    })?,
                })
            } else {
                Err(Into::<error::Error>::into(
                    error::RegValueError::SmallNameBlob,
                ))
            }
        } else {
            Err(Into::<error::Error>::into(
                error::RegValueError::SmallDataBlob,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn enumerate() {
        use crate::RegKey;
        let key =
            RegKey::open(r"\Registry\Machine\Software\Microsoft\Windows\CurrentVersion").unwrap();
        let mut iter = key.enum_values();
        assert!(iter.next().is_some());
    }
}
