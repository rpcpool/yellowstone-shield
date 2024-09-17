
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigErrors { 
    #[error("Not enough data in instructions")]
    NotEnoughData,
    
    #[error("Invalid PDA")]
    InvalidPDA
}

impl From<ConfigErrors> for ProgramError {
    fn from(e: ConfigErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}