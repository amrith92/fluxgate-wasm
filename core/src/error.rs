use thiserror::Error;

#[derive(Debug, Error)]
pub enum FluxgateError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, FluxgateError>;
