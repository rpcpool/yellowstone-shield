use std::ops::Sub;

use crate::{
    error::ConfigErrors,
    instruction::{ConfigInstructions, IndexPubkey},
    state::{AclType, ConfigListState},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh1::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction::{self, transfer},
    system_program::{self},
    sysvar::{rent::Rent, Sysvar},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ConfigInstructions::unpack(instruction_data)?;

    match instruction {
        ConfigInstructions::AddBlockList {
            blocklist,
            acl_type,
        } => add_blocklist(program_id, accounts, blocklist, acl_type)?,
        ConfigInstructions::UpdateBlocklist { edit_list } => {
            update_blocklist(program_id, accounts, edit_list)?
        }
        ConfigInstructions::CloseAccount => close_account(program_id, accounts)?,
        ConfigInstructions::UpdateAuthority => upgrade_authority(program_id, accounts)?,
        ConfigInstructions::UpdateAclType { acl_type } => {
            update_acl_type(program_id, accounts, acl_type)?
        }
    }

    Ok(())
}

fn add_blocklist(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    blocklist: Vec<Pubkey>,
    acl_type: AclType,
) -> ProgramResult {
    let (initializer, pda_config_account, system_program_account, authority) =
        check_accounts(accounts)?;

    let (pda_key, bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if pda_key.ne(pda_config_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let accounts_info = if let Some(auth) = authority {
        vec![
            initializer.clone(),
            pda_config_account.clone(),
            system_program_account.clone(),
            auth.clone(),
        ]
    } else {
        vec![
            initializer.clone(),
            pda_config_account.clone(),
            system_program_account.clone(),
        ]
    };

    let authority_key = authority.map(|val| *val.key);

    let data = ConfigListState {
        is_initialized: true,
        acl_type,
        authority: authority_key,
        blocklists: blocklist,
    };

    let space = data.get_size()?;
    let rent = get_rent(&space)?;

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_config_account.key,
            rent,
            space as u64,
            program_id,
        ),
        &accounts_info,
        &[&[
            initializer.key.as_ref(),
            "noneknows".as_bytes(),
            &[bump_seed],
        ]],
    )?;

    let mut account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;

    if account_data.is_initialized() {
        msg!("Account is already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data = data;

    account_data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

    Ok(())
}

fn update_blocklist(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    edit_list: Vec<IndexPubkey>,
) -> ProgramResult {
    let (initializer, pda_config_account, system_program_account, authority) =
        check_accounts(accounts)?;

    let (pda_key, _) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if pda_key.ne(pda_config_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let rent = Rent::get()?;

    if !rent.is_exempt(pda_config_account.lamports(), pda_config_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    let mut account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;

    if !account_data.is_initialized() {
        msg!("Account is not already initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    let authority_key = authority.map(|val| *val.key);
    if account_data.authority.is_some() && authority_key.ne(&account_data.authority) {
        msg!("Unauthorized");
        return Err(ProgramError::IncorrectAuthority);
    }

    let len = account_data.blocklists.len();

    for IndexPubkey { index, key } in edit_list {
        let index = index as usize;
        if index > len - 1 {
            account_data.blocklists.push(key);
            continue;
        }
        account_data.blocklists[index] = key;
    }

    let account_data_size = account_data.get_size()?;

    if account_data_size > pda_config_account.data_len() {
        let payment = rent.minimum_balance(account_data_size);
        let diff = payment.sub(pda_config_account.lamports());

        let accounts_info = if let Some(auth) = authority {
            vec![
                initializer.clone(),
                pda_config_account.clone(),
                system_program_account.clone(),
                auth.clone(),
            ]
        } else {
            vec![
                initializer.clone(),
                pda_config_account.clone(),
                system_program_account.clone(),
            ]
        };

        invoke(&transfer(initializer.key, &pda_key, diff), &accounts_info)?;

        pda_config_account.realloc(account_data_size, false)?;
    }

    account_data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

    Ok(())
}

fn close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let dest_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    let authority = match next_account_info(accounts_iter) {
        Ok(val) => {
            if !val.is_signer {
                msg!("Missing required signature");
                return Err(ProgramError::MissingRequiredSignature);
            }
            Some(val)
        }
        Err(ProgramError::NotEnoughAccountKeys) => None,
        Err(rest) => return Err(rest),
    };

    if system_program_account.key.ne(&system_program::ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !pda_config_account.is_writable {
        msg!("PDA account is not writable");
        return Err(ConfigErrors::NotWritableAccount.into());
    }

    let (pda_key, _) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if pda_key.ne(pda_config_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;

    if !account_data.is_initialized() {
        msg!("Account is not already initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    let authority_key = authority.map(|val| *val.key);
    if account_data.authority.is_some() && authority_key.ne(&account_data.authority) {
        msg!("Unauthorized");
        return Err(ProgramError::IncorrectAuthority);
    }

    let dest_starting_lamports = dest_account.lamports();
    **dest_account.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(pda_config_account.lamports())
        .unwrap();
    **pda_config_account.lamports.borrow_mut() = 0;

    let mut source_data = pda_config_account.data.borrow_mut();
    source_data.fill(0);

    Ok(())
}

fn upgrade_authority(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account = next_account_info(accounts_iter)?;
    let new_authority = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    let authority = match next_account_info(accounts_iter) {
        Ok(val) => {
            if !val.is_signer {
                msg!("Missing required signature");
                return Err(ProgramError::MissingRequiredSignature);
            }
            Some(val)
        }
        Err(ProgramError::NotEnoughAccountKeys) => None,
        Err(rest) => return Err(rest),
    };

    if system_program_account.key.ne(&system_program::ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !pda_config_account.is_writable {
        msg!("PDA account is not writable");
        return Err(ConfigErrors::NotWritableAccount.into());
    }

    let (pda_key, _) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if pda_key.ne(pda_config_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;

    if !account_data.is_initialized() {
        msg!("Account is not already initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    let authority_key = authority.map(|val| *val.key);
    if account_data.authority.is_some() && authority_key.ne(&account_data.authority) {
        msg!("Unauthorized");
        return Err(ProgramError::IncorrectAuthority);
    }

    account_data.authority = Some(*new_authority.key);

    account_data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

    Ok(())
}

fn update_acl_type(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    acl_type: AclType,
) -> ProgramResult {
    let (initializer, pda_config_account, ..) = check_accounts(accounts)?;

    let (pda_key, _) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if pda_key.ne(pda_config_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;

    if !account_data.is_initialized() {
        msg!("Account is not already initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    account_data.acl_type = acl_type;

    account_data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

    Ok(())
}

fn get_rent(account_len: &usize) -> Result<u64, ProgramError> {
    let rent = Rent::get()?;
    Ok(rent.minimum_balance(*account_len))
}

fn check_accounts<'a, 'b>(
    accounts: &'a [AccountInfo<'b>],
) -> Result<
    (
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        Option<&'a AccountInfo<'b>>,
    ),
    ProgramError,
>
where
    'b: 'a,
{
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    let authority = match next_account_info(accounts_iter) {
        Ok(val) => {
            if !val.is_signer {
                msg!("Missing required signature");
                return Err(ProgramError::MissingRequiredSignature);
            }
            Some(val)
        }
        Err(ProgramError::NotEnoughAccountKeys) => None,
        Err(rest) => return Err(rest),
    };

    if system_program_account.key.ne(&system_program::ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !pda_config_account.is_writable {
        msg!("PDA account is not writable");
        return Err(ConfigErrors::NotWritableAccount.into());
    }

    Ok((
        initializer,
        pda_config_account,
        system_program_account,
        authority,
    ))
}
