use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlocklistError {
    #[error("Program error: {0}")]
    ProgramError(#[from] solana_sdk::program_error::ProgramError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] std::io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type BlocklistResult<T> = Result<T, BlocklistError>;
