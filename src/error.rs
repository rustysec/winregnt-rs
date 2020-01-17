use thiserror::Error;

/// Errors produced by parsing registry
#[derive(Debug, Error)]
pub enum Error {
    /// Error open registry key
    #[error("Could not open registry key {0} error code 0x{1:08x}")]
    KeyError(String, u32),

    /// Subkey error
    #[error("Error processing sub key: {source}")]
    SubKeyError {
        /// Source of this error
        #[from]
        source: SubKeyError,
    },

    /// Registry value error
    #[error("Error processing registry value: {source}")]
    RegValueError {
        /// Source of this error
        #[from]
        source: RegValueError,
    },

    /// Converting registry data to string failed
    #[error("Could not convert registry data to string: {0}")]
    StringConversion(#[from] std::string::FromUtf16Error),
}

/// Errors encountered while processing subkeys
#[derive(Debug, Error)]
pub enum SubKeyError {
    /// Converting subkey name to string failed
    #[error("Could not convert name into string")]
    ConvertName,
}

/// Errors encountered while parsing registry values
#[derive(Debug, Error)]
pub enum RegValueError {
    /// Extracting registry value's name failed
    #[error("Could not convert name into string")]
    ConvertName,

    /// Parsing registry value's data failed
    #[error("Could not parse value data: {0}")]
    ValueData(String),

    /// The size of the name data is too small
    #[error("Name blob is too small")]
    SmallNameBlob,

    /// The size of the data field is too small
    #[error("Data blob is too small")]
    SmallDataBlob,

    /// Registry type not supported (yet)
    #[error("Encountered unsupported registry type")]
    UnknownType,

    /// Could not convert data to DWORD
    #[error("Could not convert registry data to DWORD")]
    DwordConversion,

    /// Could not read key value information
    #[error("Could not read key value full information: {0}")]
    ReadKeyValueFullInformation(#[source] std::io::Error),

    /// Could not read key information
    #[error("Could not read key basic information: {0}")]
    ReadKeyBasicInformation(#[source] std::io::Error),
}
