use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not open registry key {0} error code {1}")]
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
}
