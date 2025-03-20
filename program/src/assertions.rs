use crate::error::BlocklistError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Assert that the given account is owned by the given program or one of the given owners.
/// Useful for dealing with program interfaces.
pub fn assert_program_owner_either(
    account_name: &str,
    account: &AccountInfo,
    owners: &[Pubkey],
) -> ProgramResult {
    if !owners.iter().any(|owner| account.owner == owner) {
        msg!(
            "Account \"{}\" [{}] must be owned by either {:?}",
            account_name,
            account.key,
            owners
        );
        Err(BlocklistError::InvalidProgramOwner.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is owned by the given program.
pub fn assert_program_owner(
    account_name: &str,
    account: &AccountInfo,
    owner: &Pubkey,
) -> ProgramResult {
    if account.owner != owner {
        msg!(
            "Account \"{}\" [{}] expected program owner [{}], got [{}]",
            account_name,
            account.key,
            owner,
            account.owner
        );
        Err(BlocklistError::InvalidProgramOwner.into())
    } else {
        Ok(())
    }
}

/// Assert the derivation of the seeds against the given account and return the bump seed.
pub fn assert_pda(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(seeds, program_id);
    if *account.key != key {
        msg!(
            "Account \"{}\" [{}] is an invalid PDA. Expected the following valid PDA [{}]",
            account_name,
            account.key,
            key,
        );
        return Err(BlocklistError::InvalidPda.into());
    }
    Ok(bump)
}

/// Assert the derivation of the seeds plus bump against the given account.
pub fn assert_pda_with_bump(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds_with_bump: &[&[u8]],
) -> ProgramResult {
    let key = Pubkey::create_program_address(seeds_with_bump, program_id)?;
    if *account.key != key {
        msg!(
            "Account \"{}\" [{}] is an invalid PDA. Expected the following valid PDA [{}]",
            account_name,
            account.key,
            key,
        );
        Err(BlocklistError::InvalidPda.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is empty.
pub fn assert_empty(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        msg!(
            "Account \"{}\" [{}] must be empty",
            account_name,
            account.key,
        );
        Err(BlocklistError::ExpectedEmptyAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is non empty.
pub fn assert_non_empty(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if account.data_is_empty() {
        msg!(
            "Account \"{}\" [{}] must not be empty",
            account_name,
            account.key,
        );
        Err(BlocklistError::ExpectedNonEmptyAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is a signer.
pub fn assert_signer(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_signer {
        msg!(
            "Account \"{}\" [{}] must be a signer",
            account_name,
            account.key,
        );
        Err(BlocklistError::ExpectedSignerAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is writable.
pub fn assert_writable(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_writable {
        msg!(
            "Account \"{}\" [{}] must be writable",
            account_name,
            account.key,
        );
        Err(BlocklistError::ExpectedWritableAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account matches the given public key.
pub fn assert_same_pubkeys(
    account_name: &str,
    account: &AccountInfo,
    expected: &Pubkey,
) -> ProgramResult {
    if account.key != expected {
        msg!(
            "Account \"{}\" [{}] must match the following public key [{}]",
            account_name,
            account.key,
            expected
        );
        Err(BlocklistError::AccountMismatch.into())
    } else {
        Ok(())
    }
}

// Assert that the given amount is positive.
pub fn assert_positive_amount(
    account_name: &str,
    account: &spl_token_2022::state::Account,
) -> ProgramResult {
    if account.amount == 0 {
        msg!("Account \"{}\" must have a positive amount", account_name,);
        Err(BlocklistError::ExpectedPositiveAmount.into())
    } else {
        Ok(())
    }
}

// Assert that the given account is owned by the given token owner.
pub fn assert_token_owner(
    account_name: &str,
    expected: &Pubkey,
    account: &spl_token_2022::state::Account,
) -> ProgramResult {
    if &account.owner != expected {
        msg!(
            "Account \"{}\" owner must match the expected owner [{}]",
            account_name,
            expected
        );
        Err(BlocklistError::IncorrectTokenOwner.into())
    } else {
        Ok(())
    }
}

// Assert that the given account is associated with the given mint.
pub fn assert_mint_association(
    account_name: &str,
    expected: &Pubkey,
    account: &spl_token_2022::state::Account,
) -> ProgramResult {
    if &account.mint != expected {
        msg!(
            "Account \"{}\" mint must match the expected mint [{}]",
            account_name,
            expected
        );
        Err(BlocklistError::MistmatchMint.into())
    } else {
        Ok(())
    }
}
