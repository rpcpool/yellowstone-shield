use std::ops::Sub;

use crate::{
    error::ConfigErrors,
    instruction::{
        AclPayload, AddListPayload, ConfigInstructions, DeleteListPayload, IndexPubkey,
        InitializeListPayload,
    },
    pda::BlockList,
    state::{AclType, EnumListState, MetaList, ZEROED},
};
use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::AccountInfo,
    get_account_info,
    log::sol_log_64,
    msg,
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
    let (initializer, pda_config_account, _system_program_account, authority, bump_seed) =
        check_accounts(program_id, accounts)?;

    let pda_config_data = pda_config_account.try_borrow_data()?;
    if let Ok(_) = EnumListState::try_from_slice(&pda_config_data[..]) {
        msg!("Account already exists");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    drop(pda_config_data);

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
        BlockList::SEED_PREFIX,
        initializer.key().as_ref(),
        &[bump_seed]
    )])?;

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    data.serialize(&mut &mut pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;

    Ok(())
}

fn add_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    list: Vec<IndexPubkey>,
) -> ProgramResult {
    let (initializer, pda_config_account, _system_program_account, authority, _) =
        check_accounts(program_id, accounts)?;

    let pda_config_data = pda_config_account.try_borrow_data()?;
    let mut account_data = EnumListState::try_from_slice(&pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;
    let size = account_data.get_size()?;
    drop(pda_config_data);

    match &mut account_data {
        EnumListState::Uninitialized => {
            msg!("Account is not already initialized");
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            let mut extend_items = vec![];
            let list_len = meta_list.list_items;

            // Handle existing indexes first
            {
                let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
                for IndexPubkey { index, key } in list.iter() {
                    let index = *index as usize;
                    if (index + 1) > list_len {
                        extend_items.push(*key);
                        continue;
                    }
                    let start = size + (PUBKEY_BYTES * index);
                    let end = start + PUBKEY_BYTES;
                    pda_config_data[start..end].copy_from_slice(key);
                }
            }

            // Handle new items that need account extension
            if !extend_items.is_empty() {
                let extend_len = extend_items.len();
                let old_len = meta_list.list_items;
                meta_list.list_items += extend_len;

                let new_size = size + (extend_len * PUBKEY_BYTES);
                let payment = get_rent(&new_size)?;
                let diff = payment.sub(pda_config_account.lamports());

                if diff > 0 {
                    Transfer {
                        from: initializer,
                        to: pda_config_account,
                        lamports: diff,
                    }
                    .invoke()?;
                }

                pda_config_account.realloc(new_size, false)?;

                let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;

                // Write metadata first
                account_data
                    .serialize(&mut &mut pda_config_data[..size])
                    .map_err(|_| ProgramError::BorshIoError)?;

                // Write new items
                for (i, key) in extend_items.iter().enumerate() {
                    let start = size + (PUBKEY_BYTES * (old_len + i));
                    let end = start + PUBKEY_BYTES;
                    pda_config_data[start..end].copy_from_slice(key);
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
    msg!("Starting remove_item_list");
    msg!("Account data length");
    sol_log_64(
        0,
        0,
        0,
        0,
        pda_config_account.try_borrow_data()?.len() as u64,
    );

    // First get immutable reference to deserialize
    let pda_config_data = pda_config_account.try_borrow_data()?;
    let mut account_data = match EnumListState::try_from_slice(&pda_config_data[..]) {
        Ok(data) => {
            msg!("Successfully deserialized account data");
            data
        }
        Err(_) => {
            msg!("Failed to deserialize account data");
            return Err(ProgramError::BorshIoError);
        }
    };
    drop(pda_config_data); // Drop the immutable borrow

    let size = match account_data.get_size() {
        Ok(s) => {
            msg!("Account data size");
            sol_log_64(0, 0, 0, 0, s as u64);
            s
        }
        Err(e) => {
            msg!("Failed to get account size");
            return Err(e);
        }
    };

    match &mut account_data {
        EnumListState::Uninitialized => {
            msg!("Account is uninitialized");
            Err(ProgramError::UninitializedAccount)
        }
        EnumListState::ListStateV1(meta_list) => {
            msg!("Current list items");
            sol_log_64(0, 0, 0, 0, meta_list.list_items as u64);
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            // Now get mutable reference to modify data
            let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;

            for &index in vec_index.iter() {
                if index >= meta_list.list_items {
                    msg!("Wrong index");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let start = size + (PUBKEY_BYTES * index);
                let end = start + PUBKEY_BYTES;
                pda_config_data[start..end].copy_from_slice(&ZEROED);
            }

            // Serialize updated metadata
            account_data
                .serialize(&mut &mut pda_config_data[..size])
                .map_err(|_| ProgramError::BorshIoError)?;

            Ok(())
        }
    }
}

fn close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let initializer = get_account_info!(accounts, 0);
    let pda_config_account = get_account_info!(accounts, 1);
    let dest_account = get_account_info!(accounts, 2);
    let system_program_account = get_account_info!(accounts, 3);

    // Drop any existing borrows before assigning authority
    let authority = if accounts.len() > 4 {
        get_account_info!(accounts, 4)
    } else {
        initializer
    };

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

    // Verify PDA
    BlockList::check_pda(program_id, initializer.key(), pda_config_account.key())?;

    let pda_config_data = pda_config_account.try_borrow_data()?;
    let account_data = EnumListState::try_from_slice(&pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;
    drop(pda_config_data);

    match account_data {
        EnumListState::Uninitialized => Err(ProgramError::UninitializedAccount),
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;

            // Transfer lamports
            let mut dest_account_lamports = dest_account.try_borrow_mut_lamports()?;
            let mut pda_config_lamports = pda_config_account.try_borrow_mut_lamports()?;
            *dest_account_lamports = dest_account_lamports
                .checked_add(*pda_config_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            *pda_config_lamports = 0;

            // Clear data
            let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
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

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let mut account_data = EnumListState::try_from_slice(&pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;

    match &mut account_data {
        EnumListState::Uninitialized => {
            return Err(ProgramError::UninitializedAccount);
        }
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;
            meta_list.acl_type = acl_type;
        }
    }

    account_data
        .serialize(&mut &mut pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;

    Ok(())
}

fn freeze_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let (_initializer, pda_config_account, _, authority, _) = check_accounts(program_id, accounts)?;

    let mut pda_config_data = pda_config_account.try_borrow_mut_data()?;
    let mut account_data = EnumListState::try_from_slice(&pda_config_data[..])
        .map_err(|_| ProgramError::BorshIoError)?;

    match &mut account_data {
        EnumListState::Uninitialized => Err(ProgramError::UninitializedAccount),
        EnumListState::ListStateV1(meta_list) => {
            check_auth_freeze(&meta_list, Some(*authority.key()))?;
            meta_list.authority = None;

            // Clear and rewrite data
            pda_config_data.fill(0);
            account_data
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
    let initializer = get_account_info!(accounts, 0);
    let pda_config_account = get_account_info!(accounts, 1);
    let system_program_account = get_account_info!(accounts, 2);

    // Drop any existing borrows before assigning authority
    let authority = if accounts.len() > 3 {
        get_account_info!(accounts, 3)
    } else {
        initializer
    };

    // Verify PDA derivation before any borrows
    let (_, bump_seed) =
        BlockList::verify_pda(program_id, initializer.key(), pda_config_account.key())?;

    // Validate after PDA verification
    if !initializer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program_account.key() != &pinocchio_system::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !pda_config_account.is_writable() {
        return Err(ConfigErrors::NotWritableAccount.into());
    }

    Ok((
        initializer,
        pda_config_account,
        system_program_account,
        authority,
        bump_seed,
    ))
}
