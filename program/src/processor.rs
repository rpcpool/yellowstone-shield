use borsh::BorshDeserialize;
use bytemuck::bytes_of;
use pinocchio::instruction::Signer;
use pinocchio::memory::sol_memcpy;
use pinocchio::program_error::ProgramError;
use pinocchio::{account_info::AccountInfo, msg, pubkey::Pubkey, seeds, ProgramResult};

use crate::assertions::{
    assert_ata, assert_empty_and_owned_by_system, assert_mint_association, assert_positive_amount,
    assert_program_owner, assert_signer, assert_strategy, assert_token_owner,
    find_and_validate_pda, validate_pda,
};
use crate::error::ShieldError;
use crate::instruction::ShieldInstruction;
use crate::state::{
    Kind, PermissionStrategy, Policy, PolicyV2, Size, ZeroCopyLoad, IDENTITIES_LEN_SIZE,
};
use crate::system::{close_account, create_account, realloc_account};
use crate::BYTES_PER_PUBKEY;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction =
        ShieldInstruction::try_from_slice(instruction_data).map_err(Into::<ShieldError>::into)?;

    match instruction {
        ShieldInstruction::CreatePolicy { strategy } => {
            msg!("Instruction: Create Policy");
            create_policy(accounts, strategy)
        }
        ShieldInstruction::AddIdentity { identity } => {
            msg!("Instruction: Add Identity");
            add_identity(accounts, identity)
        }
        ShieldInstruction::RemoveIdentity { index } => {
            msg!("Instruction: Remove Identity");
            remove_identity(accounts, index)
        }
        ShieldInstruction::ReplaceIdentity { identity, index } => {
            msg!("Instruction: Replace Identity");
            replace_identity(accounts, index, identity)
        }
        ShieldInstruction::ClosePolicy => {
            msg!("Instruction: Close Policy");
            close_policy(accounts)
        }
    }
}

fn create_policy(accounts: &[AccountInfo], strategy: PermissionStrategy) -> ProgramResult {
    let [mint, token_account, policy, payer, owner, _system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert_empty_and_owned_by_system("policy", policy)?;

    validate_policy_associated_accounts(owner, mint, token_account)?;

    let strategy = strategy as u8;
    assert_strategy(strategy)?;

    let nonce = find_and_validate_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key()],
    )?;

    let record = PolicyV2 {
        kind: Kind::PolicyV2 as u8,
        strategy,
        nonce,
        mint: *mint.key(),
        identities_len: [0; 4],
    };

    let bump = &[nonce];
    let seed = seeds!(b"shield", b"policy", mint.key(), bump);
    let signer = Signer::from(&seed);

    create_account(policy, payer, PolicyV2::LEN, &crate::ID, &[signer])?;

    let mut data = policy.try_borrow_mut_data()?;

    unsafe { sol_memcpy(&mut data, bytes_of(&record), PolicyV2::LEN) };

    Ok(())
}

fn add_identity(accounts: &[AccountInfo], identity: Pubkey) -> ProgramResult {
    let [mint, token_account, policy, payer, owner, _system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    validate_policy_associated_accounts(owner, mint, token_account)?;

    let (
        identities_len_offset,
        meta_len,
        current_identities_count,
        identities_count_from_buffer,
        nonce,
    ) = {
        let data = policy.try_borrow_mut_data()?;
        match Kind::try_from(data[0])? {
            Kind::Policy => {
                let policy = unsafe { Policy::from_bytes(&data[..Policy::LEN]) }?;
                (
                    Policy::IDENTITIES_BUFFER_OFFSET,
                    Policy::LEN,
                    policy.current_identities_len(),
                    Policy::identities_len_from_buffer(data.len()),
                    policy.nonce,
                )
            }
            Kind::PolicyV2 => {
                let policy_v2 = unsafe { PolicyV2::from_bytes(&data[..PolicyV2::LEN]) }?;
                (
                    PolicyV2::IDENTITIES_BUFFER_OFFSET,
                    PolicyV2::LEN,
                    policy_v2.current_identities_len(),
                    PolicyV2::identities_len_from_buffer(data.len()),
                    policy_v2.nonce,
                )
            }
        }
    };

    validate_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key(), &[nonce]],
    )?;

    realloc_account(policy, payer, policy.data_len() + BYTES_PER_PUBKEY)?;

    let new_identity_offset = meta_len + identities_count_from_buffer * BYTES_PER_PUBKEY;

    let mut data = policy.try_borrow_mut_data()?;

    unsafe {
        sol_memcpy(
            &mut data[new_identity_offset..],
            &identity,
            BYTES_PER_PUBKEY,
        )
    };

    let updated_identities_count: [u8; IDENTITIES_LEN_SIZE] =
        (current_identities_count as u32 + 1).to_le_bytes();

    unsafe {
        sol_memcpy(
            &mut data[identities_len_offset..identities_len_offset + IDENTITIES_LEN_SIZE],
            &updated_identities_count,
            IDENTITIES_LEN_SIZE,
        )
    };

    Ok(())
}

fn remove_identity(accounts: &[AccountInfo], index: usize) -> ProgramResult {
    let [mint, token_account, policy, owner, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    validate_policy_associated_accounts(owner, mint, token_account)?;

    let mut data = policy.try_borrow_mut_data()?;

    let (identities_len_offset, meta_len, nonce, current_identities_count) =
        match Kind::try_from(data[0])? {
            Kind::Policy => {
                let policy = unsafe { Policy::from_bytes(&data[..Policy::LEN]) }?;
                (
                    Policy::IDENTITIES_BUFFER_OFFSET,
                    Policy::LEN,
                    policy.nonce,
                    policy.current_identities_len(),
                )
            }
            Kind::PolicyV2 => {
                let policy_v2 = unsafe { PolicyV2::from_bytes(&data[..PolicyV2::LEN]) }?;
                (
                    PolicyV2::IDENTITIES_BUFFER_OFFSET,
                    PolicyV2::LEN,
                    policy_v2.nonce,
                    policy_v2.current_identities_len(),
                )
            }
        };

    validate_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key(), &[nonce]],
    )?;

    let position = meta_len + index * BYTES_PER_PUBKEY;

    if position + BYTES_PER_PUBKEY > data.len() {
        return Err(ShieldError::InvalidIndexToReferenceIdentity.into());
    }

    unsafe {
        sol_memcpy(
            &mut data[position..position + BYTES_PER_PUBKEY],
            Pubkey::default().as_slice(),
            BYTES_PER_PUBKEY,
        );
    }

    let updated_identities_count: [u8; IDENTITIES_LEN_SIZE] =
        (current_identities_count as u32 - 1).to_le_bytes();

    unsafe {
        sol_memcpy(
            &mut data[identities_len_offset..identities_len_offset + IDENTITIES_LEN_SIZE],
            &updated_identities_count,
            IDENTITIES_LEN_SIZE,
        )
    };

    Ok(())
}

fn replace_identity(accounts: &[AccountInfo], index: usize, identity: Pubkey) -> ProgramResult {
    let [mint, token_account, policy, owner, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    validate_policy_associated_accounts(owner, mint, token_account)?;

    let mut data = policy.try_borrow_mut_data()?;

    let (identities_len_offset, meta_len, nonce, current_identities_count) =
        match Kind::try_from(data[0])? {
            Kind::Policy => {
                let policy = unsafe { Policy::from_bytes(&data[..Policy::LEN]) }?;
                (
                    Policy::IDENTITIES_BUFFER_OFFSET,
                    Policy::LEN,
                    policy.nonce,
                    policy.current_identities_len(),
                )
            }
            Kind::PolicyV2 => {
                let policy_v2 = unsafe { PolicyV2::from_bytes(&data[..PolicyV2::LEN]) }?;
                (
                    PolicyV2::IDENTITIES_BUFFER_OFFSET,
                    PolicyV2::LEN,
                    policy_v2.nonce,
                    policy_v2.current_identities_len(),
                )
            }
        };

    validate_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key(), &[nonce]],
    )?;

    let position = meta_len + index * BYTES_PER_PUBKEY;

    if position + BYTES_PER_PUBKEY > data.len() {
        return Err(ShieldError::InvalidIndexToReferenceIdentity.into());
    }

    let is_new_identity = data[position..position + BYTES_PER_PUBKEY] == Pubkey::default();

    unsafe {
        sol_memcpy(
            &mut data[position..position + BYTES_PER_PUBKEY],
            identity.as_ref(),
            BYTES_PER_PUBKEY,
        );
    }

    if is_new_identity {
        let updated_identities_count: [u8; IDENTITIES_LEN_SIZE] =
            (current_identities_count as u32 + 1).to_le_bytes();

        unsafe {
            sol_memcpy(
                &mut data[identities_len_offset..identities_len_offset + IDENTITIES_LEN_SIZE],
                &updated_identities_count,
                IDENTITIES_LEN_SIZE,
            )
        };
    }

    Ok(())
}

fn close_policy(accounts: &[AccountInfo]) -> ProgramResult {
    let [mint, token_account, policy, payer, owner, _system_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    validate_policy_associated_accounts(owner, mint, token_account)?;

    close_account(policy, payer)?;

    Ok(())
}

fn validate_policy_associated_accounts(
    owner: &AccountInfo,
    mint: &AccountInfo,
    token_account: &AccountInfo,
) -> ProgramResult {
    assert_signer("owner", owner)?;
    assert_program_owner("mint", mint, &spl_token_2022::id().to_bytes())?;
    assert_program_owner(
        "token_account",
        token_account,
        &spl_token_2022::ID.to_bytes(),
    )?;

    let token_account_data = &token_account.try_borrow_data()?;
    let account =
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
            token_account_data,
        )
        .map_err(Into::<ShieldError>::into)?;

    assert_ata("token_account", token_account, owner.key(), mint.key())?;
    assert_mint_association("token_account", mint.key(), &account)?;
    assert_token_owner("token_account", owner.key(), &account)?;
    assert_positive_amount("token_account", &account)?;

    Ok(())
}
