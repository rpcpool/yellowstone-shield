use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigErrors {
    #[error("Not enough data in instructions")]
    NotEnoughData,

    #[error("Invalid PDA")]
    InvalidPDA,

    #[error("The account is not writable")]
    NotWritableAccount,

    #[error("Unable to get struct size")]
    ErrorGetStructSize,

    #[error("Inmutable account")]
    ErrorInmutable,

    #[error("Incorrect authority account")]
    IncorrectAuthority,
}

impl From<ConfigErrors> for ProgramError {
    fn from(e: ConfigErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}
