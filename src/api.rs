use crate::{error::RegValueError, Result};
use std::ptr::null_mut;
use winapi::{
    shared::{
        minwindef::{DWORD, PULONG, ULONG},
        ntdef::{HANDLE, OBJECT_ATTRIBUTES, UNICODE_STRING},
    },
    um::winnt::{ACCESS_MASK, LARGE_INTEGER, PVOID},
};

/// Values read from registry keys
#[derive(Clone, Debug)]
pub enum RegValue {
    /// No value
    None,
    /// Value that can be represented as a string
    String(String),
    /// DWORD
    Dword(DWORD),
    /// QWORD
    Qword(u64),
    /// Binary data
    Binary(Vec<u8>),
    /// Unknown or unsupported registry value type
    Unknown,
}

impl ::std::fmt::Display for RegValue {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            RegValue::String(ref v) => write!(fmt, "{}", v),
            RegValue::Dword(ref v) => write!(fmt, "{}", v),
            RegValue::Qword(ref v) => write!(fmt, "{}", v),
            RegValue::Binary(ref v) => write!(fmt, "{:?}", v),
            v => write!(fmt, "? {:?}", v),
        }
    }
}

impl RegValue {
    pub(crate) fn new(info: &KeyValueFullInformation, data: &[u8]) -> Result<RegValue> {
        match info.value_type.into() {
            ValueType::REG_NONE => Ok(RegValue::None),
            ValueType::REG_SZ | ValueType::REG_EXPAND_SZ => {
                let tmp_data = data
                    .iter()
                    .copied()
                    .skip(info.data_offset as usize)
                    .take(info.data_length as usize)
                    .collect::<Vec<u8>>();
                if info.data_length > 0 && tmp_data.len() >= info.data_length as usize {
                    let wide_data = tmp_data
                        .chunks_exact(2)
                        .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                        .filter(|c| *c != 0x0000)
                        .collect::<Vec<_>>();
                    widestring::U16String::from_vec(wide_data)
                        .to_ustring()
                        .to_string()
                        .map(RegValue::String)
                        .map_err(|e| e.into())
                } else {
                    Ok(RegValue::String(String::new()))
                }
            }
            ValueType::REG_DWORD => {
                if data.len() >= std::mem::size_of::<u32>() {
                    let tmp_data = data
                        .iter()
                        .copied()
                        .skip(info.data_offset as usize)
                        .collect::<Vec<u8>>();

                    tmp_data
                        .chunks_exact(4)
                        .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .next()
                        .map(RegValue::Dword)
                        .ok_or_else(|| RegValueError::DwordConversion.into())
                } else {
                    Err(RegValueError::UnknownType.into())
                }
            }
            ValueType::REG_DWORD_BIG_ENDIAN => {
                if data.len() >= std::mem::size_of::<u32>() {
                    let tmp_data = data
                        .iter()
                        .copied()
                        .skip(info.data_offset as usize)
                        .collect::<Vec<u8>>();

                    tmp_data
                        .chunks_exact(4)
                        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .next()
                        .map(RegValue::Dword)
                        .ok_or_else(|| RegValueError::DwordConversion.into())
                } else {
                    Err(RegValueError::UnknownType.into())
                }
            }
            ValueType::REG_QWORD => {
                if data.len() >= std::mem::size_of::<u64>() {
                    let tmp_data = data
                        .iter()
                        .copied()
                        .skip(info.data_offset as usize)
                        .collect::<Vec<u8>>();

                    tmp_data
                        .chunks_exact(8)
                        .map(|chunk| {
                            u64::from_ne_bytes([
                                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5],
                                chunk[6], chunk[7],
                            ])
                        })
                        .next()
                        .map(RegValue::Qword)
                        .ok_or_else(|| RegValueError::DwordConversion.into())
                } else {
                    Err(RegValueError::UnknownType.into())
                }
            }
            ValueType::REG_BINARY => {
                let tmp_data = data
                    .iter()
                    .copied()
                    .skip(info.data_offset as usize)
                    .collect::<Vec<u8>>();

                Ok(RegValue::Binary(tmp_data))
            }
            _ => Ok(RegValue::Unknown),
        }
    }
}

/// The KEY_INFORMATION_CLASS enumeration type represents the type of information to supply about a registry key.
///
/// This library only implementes a subset of these features.
///
/// More information
/// [here](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ne-wdm-_key_information_class)
#[repr(C)]
pub enum KeyInformationClass {
    /// A KEY_BASIC_INFORMATION structure is supplied.
    KeyBasicInformation = 0,

    /// A KEY_NODE_INFORMATION structure is supplied.
    KeyNodeInformation = 1,

    /// A KEY_FULL_INFORMATION structure is supplied.
    KeyFullInformation = 2,
}

/// The KEY_VALUE_INFORMATION_CLASS enumeration type specifies the type of information to supply about the value of a registry key.
///
/// More information
/// [here](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ne-wdm-_key_value_information_class)
#[repr(C)]
pub enum KeyValueInformationClass {
    /// The information is stored as a KEY_VALUE_BASIC_INFORMATION structure.
    KeyValueBasicInformation = 0,

    /// The information is stored as a KEY_VALUE_FULL_INFORMATION structure.
    KeyValueFullInformation = 1,

    /// The information is stored as a KEY_VALUE_PARTIAL_INFORMATION structure.
    KeyValuePartialInformation = 2,

    /// The information is stored as a KEY_VALUE_FULL_INFORMATION structure that is aligned to a 64-bit (that is, 8-byte) boundary in memory. If the caller-supplied buffer does not start on a 64-bit boundary, the information is stored starting at the first 64-bit boundary in the buffer.
    KeyValueFullInformationAlign64 = 3,

    /// The information is stored as a KEY_VALUE_PARTIAL_INFORMATION structure that is aligned to a 64-bit (that is, 8-byte) boundary in memory. If the caller-supplied buffer does not start on a 64-bit boundary, the information is stored starting at the first 64-bit boundary in the buffer.
    KeyValuePartialInformationAlign64 = 4,

    /// Unspecified in MSDN documentation
    KeyValueLayerInformation = 5,

    /// The maximum value in this enumeration type.
    MaxKeyValueInfoClass = 6,
}

/// The KEY_BASIC_INFORMATION structure defines a subset of the full information that is available for a registry key.
///
/// More information
/// [here](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_key_basic_information)
#[repr(C)]
pub struct KeyBasicInformation {
    /// The last time this key or any of its values changed. This time value is expressed in absolute system time format. Absolute system time is the number of 100-nanosecond intervals since the start of the year 1601 in the Gregorian calendar.
    pub last_write_time: LARGE_INTEGER,

    /// Device and intermediate drivers should ignore this member.
    pub title_index: ULONG,

    /// An array of wide characters that contains the name of the registry key. This character string is not null-terminated. Only the first element in this array is included in the KEY_BASIC_INFORMATION structure definition. The storage for the remaining elements in the array immediately follows this element.
    pub name_length: ULONG,
    // name field comes after this
}

impl KeyBasicInformation {
    pub(crate) fn new(data: &[u8]) -> Result<Self> {
        use byteorder::{NativeEndian, ReadBytesExt};

        let mut cursor = std::io::Cursor::new(&data[std::mem::size_of::<LARGE_INTEGER>()..]);

        let this = Self {
            last_write_time: unsafe { std::mem::zeroed() },
            title_index: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyBasicInformation)?,
            name_length: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyBasicInformation)?,
        };
        Ok(this)
    }
}

/// The KEY_VALUE_FULL_INFORMATION structure defines information available for a value entry of a registry key.
///
/// More information
/// [here](https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_key_value_full_information)
#[repr(C)]
pub struct KeyValueFullInformation {
    /// Device and intermediate drivers should ignore this member.
    _title_index: ULONG,

    /// Specifies the system-defined type for the registry value(s) following the Name member. For a summary of these types, see KEY_VALUE_BASIC_INFORMATION.
    pub value_type: ULONG,

    /// Specifies the offset from the start of this structure to the data immediately following the Name string.
    pub data_offset: ULONG,

    /// Specifies the number of bytes of registry information for the value entry identified by Name.
    pub data_length: ULONG,

    /// Specifies the size in bytes of the following value entry name.
    pub name_length: ULONG,
}

impl KeyValueFullInformation {
    pub(crate) fn new(data: &[u8]) -> Result<Self> {
        use byteorder::{NativeEndian, ReadBytesExt};
        let mut cursor = std::io::Cursor::new(data);

        let this = Self {
            _title_index: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyValueFullInformation)?,
            value_type: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyValueFullInformation)?,
            data_offset: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyValueFullInformation)?,
            data_length: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyValueFullInformation)?,
            name_length: cursor
                .read_u32::<NativeEndian>()
                .map_err(RegValueError::ReadKeyValueFullInformation)?,
        };
        Ok(this)
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum ValueType {
    REG_NONE = 0,
    REG_SZ = 1,
    REG_EXPAND_SZ = 2,
    REG_BINARY = 3,
    REG_DWORD = 4,
    REG_DWORD_BIG_ENDIAN = 5,
    REG_LINK = 6,
    REG_MULTI_SZ = 7,
    REG_RESOURCE_LIST = 8,
    REG_FULL_RESOURCE_DESCRIPTOR = 9,
    REG_RESOURCE_REQUIREMENTS_LIST = 10,
    REG_QWORD = 11,
}

impl Into<ValueType> for DWORD {
    fn into(self) -> ValueType {
        match self {
            1 => ValueType::REG_SZ,
            2 => ValueType::REG_EXPAND_SZ,
            3 => ValueType::REG_BINARY,
            4 => ValueType::REG_DWORD,
            5 => ValueType::REG_DWORD_BIG_ENDIAN,
            6 => ValueType::REG_LINK,
            7 => ValueType::REG_MULTI_SZ,
            8 => ValueType::REG_RESOURCE_LIST,
            9 => ValueType::REG_FULL_RESOURCE_DESCRIPTOR,
            10 => ValueType::REG_RESOURCE_REQUIREMENTS_LIST,
            11 => ValueType::REG_QWORD,
            _ => ValueType::REG_NONE,
        }
    }
}

#[link(name = "ntdll")]
extern "system" {
    pub fn RtlInitUnicodeString(dest: *mut UNICODE_STRING, source: *const u16);
    pub fn NtEnumerateKey(
        handle: HANDLE,
        index: ULONG,
        info_class: KeyInformationClass,
        key_info: PVOID,
        length: ULONG,
        result_length: PULONG,
    ) -> u32;
    pub fn NtEnumerateValueKey(
        handle: HANDLE,
        index: ULONG,
        info_class: KeyValueInformationClass,
        key_value_info: PVOID,
        length: ULONG,
        result_length: PULONG,
    ) -> u32;
    pub fn NtClose(handle: HANDLE) -> u32;
    pub fn NtOpenKey(
        handle: *mut HANDLE,
        access: ACCESS_MASK,
        attr: *const OBJECT_ATTRIBUTES,
    ) -> u32;
    pub fn NtDeleteValueKey(handle: HANDLE, value_name: *mut UNICODE_STRING) -> u32;
    pub fn NtDeleteKey(handle: HANDLE) -> u32;
    pub fn NtSetValueKey(
        KeyHandle: HANDLE,
        ValueName: *mut UNICODE_STRING,
        TitleIndex: ULONG,
        Type: ULONG,
        Data: PVOID,
        DataSize: ULONG,
    ) -> u32;
}

pub(crate) fn enumerate_value_key(handle: HANDLE, index: ULONG) -> Option<Vec<u8>> {
    let mut result_length: ULONG = 0;
    unsafe {
        NtEnumerateValueKey(
            handle,
            index,
            KeyValueInformationClass::KeyValueFullInformation,
            null_mut() as _,
            0,
            &mut result_length,
        )
    };

    let mut data: Vec<u8> = vec![0; result_length as _];
    match unsafe {
        NtEnumerateValueKey(
            handle,
            index,
            KeyValueInformationClass::KeyValueFullInformation,
            data.as_mut_ptr() as *mut _,
            data.len() as _,
            &mut result_length,
        )
    } {
        0 => Some(data),
        _ => None,
    }
}

pub(crate) fn enumerate_key(handle: HANDLE, index: ULONG) -> Option<Vec<u8>> {
    let mut result_length: ULONG = 0;
    unsafe {
        NtEnumerateKey(
            handle,
            index,
            KeyInformationClass::KeyBasicInformation,
            null_mut() as _,
            0,
            &mut result_length,
        )
    };

    let mut data: Vec<u8> = vec![0; result_length as _];
    match unsafe {
        NtEnumerateKey(
            handle,
            index,
            KeyInformationClass::KeyBasicInformation,
            data.as_mut_ptr() as *mut _,
            data.len() as _,
            &mut result_length,
        )
    } {
        0 => Some(data),
        _ => None,
    }
}
