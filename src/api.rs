use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, PULONG, ULONG};
use winapi::shared::ntdef::{HANDLE, OBJECT_ATTRIBUTES, UNICODE_STRING};
use winapi::um::winnt::{ACCESS_MASK, LARGE_INTEGER, PVOID};

/// Values read from registry keys
#[derive(Clone, Debug)]
pub enum RegValue {
    None,
    String(String),
    Dword(DWORD),
    Unknown,
}

impl ::std::fmt::Display for RegValue {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            RegValue::String(ref v) => write!(fmt, "{}", v),
            RegValue::Dword(ref v) => write!(fmt, "{}", v),
            v => write!(fmt, "? {:?}", v),
        }
    }
}

impl RegValue {
    pub fn new(info: &KeyValueFullInformation, data: &[u8]) -> Result<RegValue, ()> {
        match info.value_type.into() {
            ValueType::REG_NONE => Ok(RegValue::None),
            ValueType::REG_SZ | ValueType::REG_EXPAND_SZ => {
                let tmp_data = data
                    .iter()
                    .copied()
                    .skip(info.data_offset as usize)
                    .collect::<Vec<u8>>();
                match info.data_length > 0 && tmp_data.len() >= info.data_length as usize {
                    true => {
                        let wstr = unsafe {
                            widestring::U16CString::from_ptr_str(tmp_data.as_ptr() as *const _)
                        };
                        wstr.to_string()
                            .map(|s| RegValue::String(s))
                            .map_err(|e| println!("to_string() failed: {}", e.to_string()))
                    }
                    false => Ok(RegValue::String(String::new())),
                }
            }
            ValueType::REG_DWORD => match data.len() >= std::mem::size_of::<u32>() {
                true => {
                    let tmp_data = data
                        .iter()
                        .copied()
                        .skip(info.data_offset as usize)
                        .collect::<Vec<u8>>();
                    let value: u32 = unsafe { std::ptr::read(tmp_data.as_ptr() as *const _) };
                    Ok(RegValue::Dword(value))
                }
                false => Err(()),
            },
            _ => Ok(RegValue::Unknown),
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
pub enum KeyInformationClass {
    KeyBasicInformation = 0,
    KeyNodeInformation = 1,
    KeyFullInformation = 2,
}

#[allow(dead_code)]
#[repr(C)]
pub enum KeyValueInformationClass {
    KeyValueBasicInformation = 0,
    KeyValueFullInformation = 1,
    KeyValuePartialInformation = 2,
    KeyValueFullInformationAlign64 = 3,
    KeyValuePartialInformationAlign64 = 4,
    KeyValueLayerInformation = 5,
    MaxKeyValueInfoClass = 6,
}

#[repr(C)]
pub struct KeyBasicInformation {
    pub last_write_time: LARGE_INTEGER,
    pub title_index: ULONG,
    pub name_length: ULONG,
    // name field comes after this
}

#[repr(C)]
pub struct KeyValueBasicInformation {
    pub title_index: ULONG,
    pub value_type: ULONG,
    pub name_length: ULONG,
    // name field comes after this
}

pub struct KeyValueFullInformation {
    pub title_length: ULONG,
    pub value_type: ULONG,
    pub data_offset: ULONG,
    pub data_length: ULONG,
    pub name_length: ULONG,
    // name field comes after this
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum ValueType {
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
