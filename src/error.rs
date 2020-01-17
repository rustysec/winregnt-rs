use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not open registry key {0} error code 0x{1:08x}")]
    KeyError(String, u32),

    #[error("Error processing sub key: {source}")]
    SubKeyError {
        #[from]
        source: SubKeyError,
    },

    #[error("Error processing registry value: {source}")]
    RegValueError {
        #[from]
        source: RegValueError,
    },

    #[error("Could not convert registry data to string: {0}")]
    StringConversion(#[from] std::string::FromUtf16Error),
}

#[derive(Debug, Error)]
pub enum SubKeyError {
    #[error("Could not convert name into string")]
    ConvertName,
}

#[derive(Debug, Error)]
pub enum RegValueError {
    #[error("Could not convert name into string")]
    ConvertName,

    #[error("Could not parse value data: {0}")]
    ValueData(String),

    #[error("Name blob is too small")]
    SmallNameBlob,

    #[error("Data blob is too small")]
    SmallDataBlob,

    #[error("Encountered unsupported registry type")]
    UnknownType,

    #[error("Could not convert registry data to DWORD")]
    DwordConversion,

    #[error("Could not read key value full information: {0}")]
    ReadKeyValueFullInformation(#[source] std::io::Error),

    #[error("Could not read key value basic information: {0}")]
    ReadKeyValueBasicInformation(#[source] std::io::Error),

    #[error("Could not read key basic information: {0}")]
    ReadKeyBasicInformation(#[source] std::io::Error),
}
