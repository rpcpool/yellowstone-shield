use std::ops::Sub;

use crate::{
    error::ConfigErrors,
    instruction::{
        AclPayload, ConfigInstructions, DeleteListPayload, EditListPayload, ExtendListPayload,
        IndexPubkey, InitializeListPayload,
    },
    pda::check_pda,
    state::{AclType, EnumListState, MetaList, ZEROED},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh1::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
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
        ConfigInstructions::InitializeList(InitializeListPayload { acl_type }) => {
            initialize_list(program_id, accounts, acl_type)?
        }
        ConfigInstructions::ExtendList(ExtendListPayload { list }) => {
            extend_list(program_id, accounts, list)?
        }
        ConfigInstructions::CloseAccount => close_account(program_id, accounts)?,
        ConfigInstructions::UpdateAclType(AclPayload { acl_type }) => {
            update_acl_type(program_id, accounts, acl_type)?
        }
        ConfigInstructions::FreezeAccount => freeze_account(program_id, accounts)?,
        ConfigInstructions::RemoveItemList(DeleteListPayload { index }) => {
            remove_item_list(program_id, accounts, index)?
        }
        ConfigInstructions::UpdateList(EditListPayload { list }) => {
            update_list(program_id, accounts, list)?
        }
    }

    Ok(())
}

fn initialize_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    acl_type: AclType,
) -> ProgramResult {
    let (initializer, pda_config_account, system_program_account, authority, bump_seed) =
        check_accounts(program_id, accounts)?;

    if let Ok(account_data) =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])
    {
        // If the account is not in the Uninitialized state, return an error
        if !matches!(account_data, EnumListState::Uninitialized) {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    }

    let mut accounts_info = vec![
        initializer.clone(),
        pda_config_account.clone(),
        system_program_account.clone(),
    ];

    if initializer.key != authority.key {
        accounts_info.push(authority.clone());
    };

    let data = EnumListState::ListStateV1(MetaList {
        acl_type,
        authority: Some(*authority.key),
        list_items: 0,
    });

    let size = data.get_size()?;

    let rent = get_rent(&size)?;

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_config_account.key,
            rent,
            size as u64,
            program_id,
        ),
        &accounts_info,
        &[&[
            initializer.key.as_ref(),
            "noneknows".as_bytes(),
            &[bump_seed],
        ]],
    )?;

    data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;
    Ok(())
}

fn extend_list(program_id: &Pubkey, accounts: &[AccountInfo], list: Vec<Pubkey>) -> ProgramResult {
    let (initializer, pda_config_account, system_program_account, authority, _) =
        check_accounts(program_id, accounts)?;

    let (pda_key, _) = check_pda(program_id, initializer.key, pda_config_account.key)?;

    let account_data =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])?;

    match account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(mut meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key))?;

            let mut accounts_info = vec![
                initializer.clone(),
                pda_config_account.clone(),
                system_program_account.clone(),
            ];

            if initializer.key != authority.key {
                accounts_info.push(authority.clone());
            };

            let list_len = list.len();
            let old_len = meta_list.list_items;
            meta_list.list_items += list_len;

            let new_size = pda_config_account.data_len() + (list_len * PUBKEY_BYTES);
            let payment = get_rent(&new_size)?;
            let diff = payment.sub(pda_config_account.lamports());

            invoke(&transfer(initializer.key, &pda_key, diff), &accounts_info)?;

            pda_config_account.realloc(new_size, false)?;
            let data = &mut &mut pda_config_account.data.borrow_mut()[..];
            EnumListState::ListStateV1(meta_list).serialize(data)?;

            for (i, pubkey) in list.iter().enumerate() {
                let start = (PUBKEY_BYTES * i) + (old_len * PUBKEY_BYTES);

                let end = start + 32;
                data[start..end].copy_from_slice(&pubkey.to_bytes());
            }
        }
    }

    Ok(())
}

fn update_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    list: Vec<IndexPubkey>,
) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let account_data =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])?;

    match &account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key))?;

            let list_len = meta_list.list_items;
            let size = account_data.get_size()?;

            let data = &mut &mut pda_config_account.data.borrow_mut()[..];

            for IndexPubkey { index, key } in list.into_iter() {
                let index: usize = index as _;
                if (index + 1) > list_len {
                    msg!("Wrong index");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let start = size + (PUBKEY_BYTES * index);

                let end = start + 32;
                data[start..end].copy_from_slice(&key.to_bytes());
            }
        }
    }

    Ok(())
}

fn remove_item_list(program_id: &Pubkey, accounts: &[AccountInfo], index: usize) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let account_data =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])?;

    match &account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key))?;

            let list_len = meta_list.list_items;
            let size = account_data.get_size()?;

            let data = &mut &mut pda_config_account.data.borrow_mut()[..];

            if (index + 1) > list_len {
                msg!("Wrong index");
                return Err(ProgramError::InvalidInstructionData);
            }
            let start = size + (PUBKEY_BYTES * index);

            let end = start + 32;
            data[start..end].copy_from_slice(&ZEROED);
            Ok(())
        }
    }
}

fn close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account = next_account_info(accounts_iter)?;
    let dest_account = next_account_info(accounts_iter)?;

    let system_program_account = next_account_info(accounts_iter)?;

    let authority = match next_account_info(accounts_iter) {
        Ok(val) => {
            if !val.is_signer {
                msg!("Missing required signature");
                return Err(ProgramError::MissingRequiredSignature);
            }
            val
        }
        Err(ProgramError::NotEnoughAccountKeys) => initializer,
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

    check_pda(program_id, initializer.key, pda_config_account.key)?;

    let account_data =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])?;

    match &account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key))?;

            let dest_starting_lamports = dest_account.lamports();
            **dest_account.lamports.borrow_mut() = dest_starting_lamports
                .checked_add(pda_config_account.lamports())
                .unwrap();
            **pda_config_account.lamports.borrow_mut() = 0;

            let mut source_data = pda_config_account.data.borrow_mut();
            source_data.fill(0);
            Ok(())
        }
    }
}

fn update_acl_type(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    acl_type: AclType,
) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let rent = Rent::get()?;

    if !rent.is_exempt(pda_config_account.lamports(), pda_config_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    let account_data =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])?;

    match account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(mut meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key))?;

            meta_list.acl_type = acl_type;

            EnumListState::ListStateV1(meta_list)
                .serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

            Ok(())
        }
    }
}

fn freeze_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let account_data =
        try_from_slice_unchecked::<EnumListState>(&pda_config_account.data.borrow()[..])?;

    match account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(mut meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key))?;

            meta_list.authority = None;

            EnumListState::ListStateV1(meta_list)
                .serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

            Ok(())
        }
    }
}

fn get_rent(account_len: &usize) -> Result<u64, ProgramError> {
    let rent = Rent::get()?;
    Ok(rent.minimum_balance(*account_len))
}

fn check_auth_freeze(meta_list: &MetaList, auth_key: Option<Pubkey>) -> ProgramResult {
    if meta_list.authority.is_none() {
        msg!("Account is freeze");
        return Err(ConfigErrors::ErrorInmutable.into());
    }

    if auth_key.ne(&meta_list.authority) {
        msg!("Unauthorized");
        return Err(ProgramError::IncorrectAuthority);
    }
    Ok(())
}

fn check_accounts<'a, 'b>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<
    (
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        &'a AccountInfo<'b>,
        u8,
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
            val
        }
        Err(ProgramError::NotEnoughAccountKeys) => initializer,
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

    let (_, bump_seed) = check_pda(program_id, initializer.key, pda_config_account.key)?;

    Ok((
        initializer,
        pda_config_account,
        system_program_account,
        authority,
        bump_seed,
    ))
}
