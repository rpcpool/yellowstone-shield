use num_derive::FromPrimitive;
use pinocchio::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum ShieldError {
    /// 0 - Error deserializing an account
    #[error("Error deserializing an account")]
    DeserializationError,
    /// 1 - Error serializing an account
    #[error("Error serializing an account")]
    SerializationError,
    /// 2 - Invalid program owner
    #[error("Invalid program owner. This likely mean the provided account does not exist")]
    InvalidProgramOwner,
    /// 3 - Invalid PDA derivation
    #[error("Invalid PDA derivation")]
    InvalidPda,
    /// 4 - Expected empty account
    #[error("Expected empty account")]
    ExpectedEmptyAccount,
    /// 5 - Expected non empty account
    #[error("Expected non empty account")]
    ExpectedNonEmptyAccount,
    /// 6 - Expected signer account
    #[error("Expected signer account")]
    ExpectedSignerAccount,
    /// 7 - Expected writable account
    #[error("Expected writable account")]
    ExpectedWritableAccount,
    /// 8 - Account mismatch
    #[error("Account mismatch")]
    AccountMismatch,
    /// 9 - Invalid account key
    #[error("Invalid account key")]
    InvalidAccountKey,
    /// 10 - Numerical overflow
    #[error("Numerical overflow")]
    NumericalOverflow,
    /// 11 - Expected postive token account amount
    #[error("Expected ositive amount")]
    ExpectedPositiveAmount,
    /// 12 - Incorrect token owner
    #[error("Incorrect token owner")]
    IncorrectTokenOwner,
    /// 13 - Mismatching mint
    #[error("Mismatching mint")]
    MistmatchMint,
    /// 14 - identity not found
    #[error("identity not found")]
    IdentityNotFound,
    /// 15 - Invalid associated token account
    #[error("Invalid associated token account")]
    InvalidAssociatedTokenAccount,
    /// 16 - Condition not met
    #[error("Condition not met")]
    MissedCondition,
    /// 17 - Invalid account data
    #[error("invalid account data")]
    InvalidAccountData,
    /// 18 - Invalid argument
    #[error("Invalid argument")]
    InvalidArgument,
    /// 19 - Invalid instruction data
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    /// 20 - Account data too small
    #[error("Account data too small")]
    AccountDataTooSmall,
    /// 21 - Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
    /// 22 - Incorrect program id
    #[error("Incorrect program id")]
    IncorrectProgramId,
    /// 23 - Missing required signature
    #[error("Missing required signature")]
    MissingRequiredSignature,
    /// 24 - Account already initialized
    #[error("Account already initialized")]
    AccountAlreadyInitialized,
    /// 25 - Uninitialized account
    #[error("Uninitialized account")]
    UninitializedAccount,
    /// 26 - Not enough account keys
    #[error("Not enough account keys")]
    NotEnoughAccountKeys,
    /// 27 - Account borrow failed
    #[error("Account borrow failed")]
    AccountBorrowFailed,
    /// 28 - Max seed length exceeded
    #[error("Max seed length exceeded")]
    MaxSeedLengthExceeded,
    /// 29 - Invalid seeds
    #[error("Invalid seeds")]
    InvalidSeeds,
    /// 30 - Borsh IO error
    #[error("Borsh IO error")]
    BorshIoError,
    /// 31 - Account not rent exempt
    #[error("Account not rent exempt")]
    AccountNotRentExempt,
    /// 32 - Unsupported sysvar
    #[error("Unsupported sysvar")]
    UnsupportedSysvar,
    /// 33 - Illegal owner
    #[error("Illegal owner")]
    IllegalOwner,
    /// 34 - Max accounts data allocations exceeded
    #[error("Max accounts data allocations exceeded")]
    MaxAccountsDataAllocationsExceeded,
    /// 35 - Invalid realloc
    #[error("Invalid realloc")]
    InvalidRealloc,
    /// 36 - Max instruction trace length exceeded
    #[error("Max instruction trace length exceeded")]
    MaxInstructionTraceLengthExceeded,
    /// 37 - Builtin programs must consume compute units
    #[error("Builtin programs must consume compute units")]
    BuiltinProgramsMustConsumeComputeUnits,
    /// 38 - Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner,
    /// 39 - Arithmetic overflow
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,
    /// 40 - Immutable
    #[error("Immutable")]
    Immutable,
    /// 41 - Incorrect authority
    #[error("Incorrect authority")]
    IncorrectAuthority,
    /// 42 - Generic error
    #[error("Generic program error")]
    GenericError,
    // 43 - Invalid strategy
    #[error("Invalid strategy")]
    InvalidStrategy,
    // 44 - Invalid Policy Kind
    #[error("Invalid Policy Kind")]
    InvalidPolicyKind,
    // 45 - Invalid Index To Reference Identity
    #[error("Invalid Index To Reference Identity")]
    InvalidIndexToReferenceIdentity,
}

impl From<std::io::Error> for ShieldError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::InvalidData => Self::DeserializationError,
            std::io::ErrorKind::UnexpectedEof => Self::SerializationError,
            _ => Self::GenericError,
        }
    }
}

impl From<solana_program::program_error::ProgramError> for ShieldError {
    fn from(e: solana_program::program_error::ProgramError) -> Self {
        match e {
            solana_program::program_error::ProgramError::InvalidArgument => Self::InvalidArgument,
            solana_program::program_error::ProgramError::InvalidInstructionData => {
                Self::InvalidInstructionData
            }
            solana_program::program_error::ProgramError::InvalidAccountData => {
                Self::InvalidAccountData
            }
            solana_program::program_error::ProgramError::AccountDataTooSmall => {
                Self::AccountDataTooSmall
            }
            solana_program::program_error::ProgramError::InsufficientFunds => {
                Self::InsufficientFunds
            }
            solana_program::program_error::ProgramError::IncorrectProgramId => {
                Self::IncorrectProgramId
            }
            solana_program::program_error::ProgramError::MissingRequiredSignature => {
                Self::MissingRequiredSignature
            }
            solana_program::program_error::ProgramError::AccountAlreadyInitialized => {
                Self::AccountAlreadyInitialized
            }
            solana_program::program_error::ProgramError::UninitializedAccount => {
                Self::UninitializedAccount
            }
            solana_program::program_error::ProgramError::NotEnoughAccountKeys => {
                Self::NotEnoughAccountKeys
            }
            solana_program::program_error::ProgramError::AccountBorrowFailed => {
                Self::AccountBorrowFailed
            }
            solana_program::program_error::ProgramError::MaxSeedLengthExceeded => {
                Self::MaxSeedLengthExceeded
            }
            solana_program::program_error::ProgramError::InvalidSeeds => Self::InvalidSeeds,
            solana_program::program_error::ProgramError::BorshIoError(_) => Self::BorshIoError,
            solana_program::program_error::ProgramError::AccountNotRentExempt => {
                Self::AccountNotRentExempt
            }
            solana_program::program_error::ProgramError::UnsupportedSysvar => {
                Self::UnsupportedSysvar
            }
            solana_program::program_error::ProgramError::IllegalOwner => Self::IllegalOwner,
            solana_program::program_error::ProgramError::MaxAccountsDataAllocationsExceeded => {
                Self::MaxAccountsDataAllocationsExceeded
            }
            solana_program::program_error::ProgramError::InvalidRealloc => Self::InvalidRealloc,
            solana_program::program_error::ProgramError::MaxInstructionTraceLengthExceeded => {
                Self::MaxInstructionTraceLengthExceeded
            }
            solana_program::program_error::ProgramError::BuiltinProgramsMustConsumeComputeUnits => {
                Self::BuiltinProgramsMustConsumeComputeUnits
            }
            solana_program::program_error::ProgramError::InvalidAccountOwner => {
                Self::InvalidAccountOwner
            }
            solana_program::program_error::ProgramError::ArithmeticOverflow => {
                Self::ArithmeticOverflow
            }
            solana_program::program_error::ProgramError::Immutable => Self::Immutable,
            solana_program::program_error::ProgramError::IncorrectAuthority => {
                Self::IncorrectAuthority
            }
            solana_program::program_error::ProgramError::Custom(_) => Self::GenericError,
        }
    }
}

impl From<ShieldError> for ProgramError {
    fn from(e: ShieldError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
