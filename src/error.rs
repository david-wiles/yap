use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Password {name} not found in this vault")]
    PasswordNotFound { name: String },

    #[error("No home directory was found, could not process request")]
    NoHomeDir,

    #[error("Config key {key} does not exist")]
    BadConfigKey { key: String },

    #[error("Error during encryption or decryption: {0}")]
    CryptoError(#[from] ring::error::Unspecified),

    #[error("IO Error: {0}")]
    StdIO(#[from] std::io::Error),

    #[error("Unable to serialize or deserialize: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        e.to_string()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
