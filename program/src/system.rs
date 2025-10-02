use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::{CreateAccount, Transfer};

use crate::error::ShieldError;

/// Create a new account from the given size.
#[inline(always)]
pub fn create_account(
    to: &AccountInfo,
    from: &AccountInfo,
    size: usize,
    owner: &Pubkey,
    signers: &[Signer],
) -> ProgramResult {
    let rent = Rent::get()?;
    let lamports: u64 = rent.minimum_balance(size);
    let space = size as u64;

    CreateAccount {
        from,
        to,
        lamports,
        space,
        owner,
    }
    .invoke_signed(signers)?;

    Ok(())
}

/// Resize an account using realloc, lifted from Solana Cookbook.
#[inline(always)]
pub fn realloc_account(
    target_account: &AccountInfo,
    funding_account: &AccountInfo,
    new_size: usize,
) -> ProgramResult {
    let rent = Rent::get()?;
    let old_minimum_balance = rent.minimum_balance(target_account.data_len());
    let new_minimum_balance = rent.minimum_balance(new_size);
    let lamports_diff = new_minimum_balance.abs_diff(old_minimum_balance);

    if new_minimum_balance > old_minimum_balance {
        Transfer {
            from: funding_account,
            to: target_account,
            lamports: lamports_diff,
        }
        .invoke()?;
    } else {
        transfer_lamports_from_pdas(target_account, funding_account, lamports_diff)?;
    }

    target_account.realloc(new_size, false)
}

// /// Close an account.
#[inline(always)]
pub fn close_account(
    target_account: &AccountInfo,
    receiving_account: &AccountInfo,
) -> ProgramResult {
    let dest_starting_lamports = receiving_account.lamports();
    *receiving_account.try_borrow_mut_lamports()? = dest_starting_lamports
        .checked_add(target_account.lamports())
        .ok_or(ShieldError::NumericalOverflow)?;

    *target_account.try_borrow_mut_lamports()? = 0;

    unsafe {
        target_account.assign(&pinocchio_system::ID);
    }
    target_account.realloc(0, false)
}

pub fn transfer_lamports_from_pdas(
    from: &AccountInfo,
    to: &AccountInfo,
    lamports: u64,
) -> ProgramResult {
    let from_lamports = from.try_borrow_mut_lamports()?;

    from_lamports
        .checked_sub(lamports)
        .ok_or(ShieldError::NumericalOverflow)?;

    let to_lamports = to.try_borrow_mut_lamports()?;

    to_lamports
        .checked_add(lamports)
        .ok_or(ShieldError::NumericalOverflow)?;

    Ok(())
}
