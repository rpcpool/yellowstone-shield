use std::ops::Sub;

use crate::{
    error::ConfigErrors,
    instruction::{
        AclPayload, AddListPayload, ConfigInstructions, DeleteListPayload, IndexPubkey,
        InitializeListPayload,
    },
    pda::check_pda,
    state::{AclType, EnumListState, MetaList, ZEROED},
};
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::AccountInfo,
    get_account_info, msg,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
    signer,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::{CreateAccount, Transfer};

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
        ConfigInstructions::Add(AddListPayload { list }) => add_list(program_id, accounts, list)?,
        ConfigInstructions::CloseAccount => close_account(program_id, accounts)?,
        ConfigInstructions::UpdateAclType(AclPayload { acl_type }) => {
            update_acl_type(program_id, accounts, acl_type)?
        }
        ConfigInstructions::FreezeAccount => freeze_account(program_id, accounts)?,
        ConfigInstructions::RemoveItemList(DeleteListPayload { vec_index }) => {
            remove_item_list(program_id, accounts, vec_index)?
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

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    if let Ok(_) = EnumListState::deserialize(&mut &pda_config_data[..]) {
        msg!("Account already exists");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let mut accounts_info = vec![
        initializer.clone(),
        pda_config_account.clone(),
        system_program_account.clone(),
    ];

    if initializer.key() != authority.key() {
        accounts_info.push(authority.clone());
    };

    let data = EnumListState::ListStateV1(MetaList {
        acl_type,
        authority: Some(*authority.key()),
        list_items: 0,
    });

    let size = data.get_size()?;

    let rent = get_rent(&size)?;

    CreateAccount {
        from: initializer,
        to: pda_config_account,
        lamports: rent,
        space: size as u64,
        owner: program_id,
    }
    .invoke_signed(&[signer!(
        initializer.key().as_ref(),
        b"noneknows",
        &[bump_seed]
    )])?;

    data.serialize(&mut &mut pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;

    Ok(())
}

fn add_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    list: Vec<IndexPubkey>,
) -> ProgramResult {
    let (initializer, pda_config_account, system_program_account, authority, _) =
        check_accounts(program_id, accounts)?;

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let account_data = match EnumListState::deserialize(&mut &pda_config_data[..]) {
        Ok(data) => data,
        Err(_) => {
            msg!("Account is not already initialized or does not exist");
            return Err(ProgramError::UninitializedAccount);
        }
    };
    let size = account_data.get_size()?;

    match account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(mut meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            let mut accounts_info = vec![
                initializer.clone(),
                pda_config_account.clone(),
                system_program_account.clone(),
            ];

            if initializer.key() != authority.key() {
                accounts_info.push(authority.clone());
            };

            let mut extend_items = vec![];
            let list_len = meta_list.list_items;

            {
                for IndexPubkey { index, key } in list.into_iter() {
                    let index: usize = index as _;
                    if (index + 1) > list_len {
                        extend_items.push(key);
                        continue;
                    }
                    let start = size + (PUBKEY_BYTES * index);

                    let end = start + PUBKEY_BYTES;
                    pda_config_data[start..end].copy_from_slice(&key);
                }
            }

            if !extend_items.is_empty() {
                let extend_len = extend_items.len();

                let old_len = meta_list.list_items;
                meta_list.list_items += extend_len;

                let new_size = pda_config_account.data_len() + (extend_len * PUBKEY_BYTES);
                let payment = get_rent(&new_size)?;
                let diff = payment.sub(pda_config_account.lamports());

                Transfer {
                    from: initializer,
                    to: pda_config_account,
                    lamports: diff,
                }
                .invoke()
                .map_err(|e| e)?;

                pda_config_account.realloc(new_size, false)?;
                EnumListState::ListStateV1(meta_list.clone())
                    .serialize(&mut &mut pda_config_data[..])
                    .map_err(|_| ProgramError::BorshIoError)?;

                for (i, pubkey) in extend_items.iter().enumerate() {
                    let start = (PUBKEY_BYTES * i) + (old_len * PUBKEY_BYTES);

                    let end = start + 32;
                    pda_config_data[start..end].copy_from_slice(pubkey);
                }
            }
        }
    }

    Ok(())
}

fn remove_item_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vec_index: Vec<usize>,
) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let account_data = match EnumListState::deserialize(&mut &pda_config_data[..]) {
        Ok(data) => data,
        Err(_) => {
            msg!("Account is not already initialized or does not exist");
            return Err(ProgramError::UninitializedAccount);
        }
    };

    match &account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            let list_len = meta_list.list_items;
            let size = account_data.get_size()?;

            for index in vec_index.into_iter() {
                if (index + 1) > list_len {
                    msg!("Wrong index");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let start = size + (PUBKEY_BYTES * index);

                let end = start + PUBKEY_BYTES;
                pda_config_data[start..end].copy_from_slice(&ZEROED);
            }

            Ok(())
        }
    }
}

fn close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let initializer = get_account_info!(accounts, 0 as usize);
    let pda_config_account = get_account_info!(accounts, 1 as usize);
    let dest_account = get_account_info!(accounts, 2 as usize);
    let system_program_account = get_account_info!(accounts, 3 as usize);
    let authority = get_account_info!(accounts, 4 as usize);

    if authority.is_signer() {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program_account.key().ne(&pinocchio_system::id()) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !initializer.is_signer() {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !pda_config_account.is_writable() {
        msg!("PDA account is not writable");
        return Err(ConfigErrors::NotWritableAccount.into());
    }

    check_pda(program_id, initializer.key(), pda_config_account.key())?;

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let account_data = match EnumListState::deserialize(&mut &pda_config_data[..]) {
        Ok(data) => data,
        Err(_) => {
            msg!("Account is not already initialized or does not exist");
            return Err(ProgramError::UninitializedAccount);
        }
    };

    match &account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            let mut dest_account_lamports = dest_account.try_borrow_mut_lamports()?;
            let mut pda_config_lamports = pda_config_account.try_borrow_mut_lamports()?;
            *dest_account_lamports = dest_account_lamports
                .checked_add(*pda_config_lamports)
                .unwrap();
            *pda_config_lamports = 0;

            pda_config_data.fill(0);
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

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let account_data = match EnumListState::deserialize(&mut &pda_config_data[..]) {
        Ok(data) => data,
        Err(_) => {
            msg!("Account is not already initialized or does not exist");
            return Err(ProgramError::UninitializedAccount);
        }
    };

    match account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(mut meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            meta_list.acl_type = acl_type;

            EnumListState::ListStateV1(meta_list)
                .serialize(&mut &mut pda_config_data[..])
                .map_err(|_| ProgramError::BorshIoError)?;

            Ok(())
        }
    }
}

fn freeze_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let account_data = match EnumListState::deserialize(&mut &pda_config_data[..]) {
        Ok(data) => data,
        Err(_) => {
            msg!("Account is not already initialized or does not exist");
            return Err(ProgramError::UninitializedAccount);
        }
    };

    match account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(mut meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            meta_list.authority = None;

            EnumListState::ListStateV1(meta_list)
                .serialize(&mut &mut pda_config_data[..])
                .map_err(|_| ProgramError::BorshIoError)?;

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
        msg!("Account is frozen");
        return Err(ConfigErrors::ErrorInmutable.into());
    }

    if auth_key.ne(&meta_list.authority) {
        msg!("Unauthorized");
        return Err(ConfigErrors::IncorrectAuthority.into());
    }
    Ok(())
}

fn check_accounts<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo],
) -> Result<
    (
        &'a AccountInfo,
        &'a AccountInfo,
        &'a AccountInfo,
        &'a AccountInfo,
        u8,
    ),
    ProgramError,
> {
    let initializer = get_account_info!(accounts, 0 as usize);
    let pda_config_account = get_account_info!(accounts, 1 as usize);
    let system_program_account = get_account_info!(accounts, 2 as usize);
    let authority = get_account_info!(accounts, 3 as usize);

    if !authority.is_signer() {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program_account.key().ne(&pinocchio_system::id()) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !initializer.is_signer() {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !pda_config_account.is_writable() {
        msg!("PDA account is not writable");
        return Err(ConfigErrors::NotWritableAccount.into());
    }

    let (_, bump_seed) = check_pda(program_id, initializer.key(), pda_config_account.key())?;

    Ok((
        initializer,
        pda_config_account,
        system_program_account,
        authority,
        bump_seed,
    ))
}
