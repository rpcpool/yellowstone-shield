use crate::error::ShieldError;
use pinocchio::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    pubkey::{create_program_address, find_program_address, Pubkey},
    ProgramResult,
};
use spl_token_2022::extension::StateWithExtensions;

/// Assert that the given strategy is valid.
pub fn assert_strategy(strategy: u8) -> ProgramResult {
    if strategy > 1 {
        return Err(ShieldError::InvalidStrategy.into());
    }

    Ok(())
}

/// Assert that the given account is owned by the given program.
pub fn assert_program_owner(
    account_name: &str,
    account: &AccountInfo,
    owner: &Pubkey,
) -> ProgramResult {
    if account.is_owned_by(owner) {
        return Ok(());
    }
    msg!(
        "Account \"{}\" [{:?}] expected program owner [{:?}], got [{:?}]",
        account_name,
        account.key(),
        owner,
        unsafe { account.owner() },
    );
    Err(ShieldError::InvalidProgramOwner.into())
}

/// Assert the derivation of the seeds against the given account and return the bump seed.
pub fn find_and_validate_pda(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = find_program_address(seeds, program_id);
    if *account.key() != key {
        msg!(
            "Account \"{}\" [{:?}] is an invalid PDA. Expected the following valid PDA [{:?}]",
            account_name,
            account.key(),
            key,
        );
        return Err(ShieldError::InvalidPda.into());
    }
    Ok(bump)
}

pub fn validate_pda(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let key = create_program_address(seeds, program_id)?;
    if *account.key() != key {
        msg!(
            "Account \"{}\" [{:?}] is an invalid PDA. Expected the following valid PDA [{:?}]",
            account_name,
            account.key(),
            key,
        );
        return Err(ShieldError::InvalidPda.into());
    }

    Ok(())
}

/// Assert a condition and return an error if it is not met.
pub fn assert_condition(condition: bool, msg: &str) -> ProgramResult {
    if condition {
        return Ok(());
    }

    msg!(msg);
    Err(ShieldError::MissedCondition.into())
}

/// Assert the derivation of the seeds plus bump against the given account.
pub fn assert_pda_with_bump(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds_with_bump: &[&[u8]],
) -> ProgramResult {
    let key = create_program_address(seeds_with_bump, program_id)?;
    if *account.key() != key {
        msg!(
            "Account \"{}\" [{:?}] is an invalid PDA. Expected the following valid PDA [{:?}]",
            account_name,
            account.key(),
            key,
        );
        Err(ShieldError::InvalidPda.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is empty.
pub fn assert_empty_and_owned_by_system(
    account_name: &str,
    account: &AccountInfo,
) -> ProgramResult {
    if !(account.data_is_empty() && unsafe { account.owner() } == &pinocchio_system::ID) {
        msg!(
            "Account \"{}\" [{:?}] must be empty and owned by system_program",
            account_name,
            account.key(),
        );
        Err(ShieldError::ExpectedEmptyAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is non empty.
pub fn assert_non_empty(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if account.data_is_empty() {
        msg!(
            "Account \"{}\" [{:?}] must not be empty",
            account_name,
            account.key(),
        );
        Err(ShieldError::ExpectedNonEmptyAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is a signer.
pub fn assert_signer(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_signer() {
        msg!(
            "Account \"{}\" [{:?}] must be a signer",
            account_name,
            account.key(),
        );
        Err(ShieldError::ExpectedSignerAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is writable.
pub fn assert_writable(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_writable() {
        msg!(
            "Account \"{}\" [{:?}] must be writable",
            account_name,
            account.key(),
        );
        Err(ShieldError::ExpectedWritableAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is writable and signer
pub fn assert_writable_and_signer(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_writable() {
        msg!(
            "Account \"{}\" [{:?}] must be writable",
            account_name,
            account.key(),
        );
        return Err(ShieldError::ExpectedWritableAccount.into());
    }

    if !account.is_signer() {
        msg!(
            "Account \"{}\" [{:?}] must be a signer",
            account_name,
            account.key(),
        );
        return Err(ShieldError::ExpectedSignerAccount.into());
    }

    Ok(())
}

/// Assert that the given account matches the given public key.
pub fn assert_same_pubkeys(
    account_name: &str,
    account: &AccountInfo,
    expected: &Pubkey,
) -> ProgramResult {
    if account.key() != expected {
        msg!(
            "Account \"{}\" [{:?}] must match the following public key [{:?}]",
            account_name,
            account.key(),
            expected
        );
        Err(ShieldError::AccountMismatch.into())
    } else {
        Ok(())
    }
}

// Assert that the given amount is positive.
pub fn assert_positive_amount(
    account_name: &str,
    account: &StateWithExtensions<spl_token_2022::state::Account>,
) -> ProgramResult {
    if account.base.amount == 0 {
        msg!("Account \"{}\" must have a positive amount", account_name,);
        Err(ShieldError::ExpectedPositiveAmount.into())
    } else {
        Ok(())
    }
}

// Assert that the given account is owned by the given token owner.
pub fn assert_token_owner(
    account_name: &str,
    expected: &Pubkey,
    account: &StateWithExtensions<spl_token_2022::state::Account>,
) -> ProgramResult {
    if *expected != account.base.owner.to_bytes() {
        msg!(
            "Account \"{}\" owner must match the expected owner [{:?}]",
            account_name,
            expected
        );
        Err(ShieldError::IncorrectTokenOwner.into())
    } else {
        Ok(())
    }
}

// Assert that the given account is associated with the given mint.
pub fn assert_mint_association(
    account_name: &str,
    expected: &Pubkey,
    account: &StateWithExtensions<spl_token_2022::state::Account>,
) -> ProgramResult {
    if &account.base.mint.to_bytes() != expected {
        msg!(
            "Account \"{}\" mint must match the expected mint [{:?}]",
            account_name,
            expected
        );
        Err(ShieldError::MistmatchMint.into())
    } else {
        Ok(())
    }
}

pub fn assert_ata(
    account_name: &str,
    account: &AccountInfo,
    owner: &Pubkey,
    mint: &Pubkey,
) -> ProgramResult {
    let (ata, _) = find_program_address(
        &[owner, &spl_token_2022::ID.to_bytes(), mint],
        &spl_associated_token_account::ID.to_bytes(),
    );
    if account.key() != &ata {
        msg!(
            "Account \"{}\" [{:?}] must be the associated token account for [{:?}]",
            account_name,
            account.key(),
            ata
        );
        Err(ShieldError::InvalidAssociatedTokenAccount.into())
    } else {
        Ok(())
    }
}
