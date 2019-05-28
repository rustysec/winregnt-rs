use winapi::shared::minwindef::{DWORD, PULONG, ULONG};
use winapi::shared::ntdef::{HANDLE, OBJECT_ATTRIBUTES, UNICODE_STRING};
use winapi::um::winnt::{ACCESS_MASK, LARGE_INTEGER, PVOID};

/// Values read from registry keys
pub enum RegValue {
    String(String),
    Dword(DWORD),
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

#[allow(non_camel_case_types)]
#[repr(C)]
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
