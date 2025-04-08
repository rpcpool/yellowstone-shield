use borsh::BorshDeserialize;
use bytemuck::bytes_of;
use pinocchio::instruction::Signer;
use pinocchio::memory::sol_memcpy;
use pinocchio::{account_info::AccountInfo, msg, pubkey::Pubkey, seeds, ProgramResult};
use solana_program::system_program;

use crate::assertions::{
    assert_ata, assert_condition, assert_empty, assert_mint_association, assert_pda,
    assert_positive_amount, assert_program_owner, assert_same_pubkeys, assert_signer,
    assert_strategy, assert_token_owner, assert_writable,
};
use crate::error::ShieldError;
use crate::instruction::ShieldInstruction;
use crate::state::{PermissionStrategy, Policy, Size, ZeroCopyLoad};
use crate::system::{close_account, create_account, realloc_account};

const BYTES_PER_PUBKEY: usize = core::mem::size_of::<Pubkey>();

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
    let mint = &accounts[0];
    let token_account = &accounts[1];
    let policy = &accounts[2];
    let payer = &accounts[3];
    let owner = &accounts[4];
    let system_program = &accounts[5];

    let nonce = assert_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key()],
    )?;

    assert_same_pubkeys(
        "system_program",
        system_program,
        &system_program::ID.to_bytes(),
    )?;
    assert_signer("payer", payer)?;
    assert_signer("owner", owner)?;
    assert_writable("payer", payer)?;
    assert_writable("policy", policy)?;
    assert_ata("token_account", token_account, &owner.key(), &mint.key())?;
    assert_program_owner("mint", mint, &spl_token_2022::ID.to_bytes())?;
    assert_program_owner(
        "token_account",
        &token_account,
        &spl_token_2022::ID.to_bytes(),
    )?;

    let token_account_data = &token_account.try_borrow_data()?;
    let account =
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
            token_account_data,
        )
        .map_err(Into::<ShieldError>::into)?;

    assert_positive_amount("token_account", &account)?;
    assert_token_owner("token_account", &owner.key(), &account)?;
    assert_mint_association("token_account", &mint.key(), &account)?;
    assert_empty("policy", &policy)?;

    let strategy = strategy as u8;
    assert_strategy(strategy)?;

    let record = Policy {
        kind: 0,
        strategy,
        nonce,
        identities_len: [0; 4],
    };

    let bump = &[nonce];
    let seed = seeds!(b"shield", b"policy", mint.key(), bump);
    let signer = Signer::from(&seed);

    create_account(&policy, &payer, Policy::LEN, &crate::ID, &[signer])?;

    let mut data = policy.try_borrow_mut_data()?;

    unsafe { sol_memcpy(&mut data, bytes_of(&record), Policy::LEN) };

    Ok(())
}

fn add_identity(accounts: &[AccountInfo], identity: Pubkey) -> ProgramResult {
    let mint = &accounts[0];
    let token_account = &accounts[1];
    let policy = &accounts[2];
    let payer = &accounts[3];
    let owner = &accounts[4];
    let system_program = &accounts[5];

    let record = unsafe { Policy::load(policy)? };

    let bump = assert_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key()],
    )?;

    assert_condition(bump == record.nonce, "Policy nonce mismatch")?;
    assert_same_pubkeys("system_program", system_program, &pinocchio_system::ID)?;
    assert_signer("payer", payer)?;
    assert_signer("owner", owner)?;
    assert_writable("payer", payer)?;
    assert_writable("policy", policy)?;
    assert_program_owner("mint", mint, &spl_token_2022::ID.to_bytes())?;
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

    assert_positive_amount("token_account", &account)?;
    assert_ata("token_account", token_account, &owner.key(), &mint.key())?;
    assert_token_owner("token_account", &owner.key(), &account)?;
    assert_mint_association("token_account", &mint.key(), &account)?;

    realloc_account(policy, payer, policy.data_len() + BYTES_PER_PUBKEY)?;

    let mut data = policy.try_borrow_mut_data()?;
    let policy_metadata = &data[..Policy::LEN];

    let policy = unsafe { Policy::from_bytes(policy_metadata) };

    let current_identities_count = policy.identities_len();
    let new_identity_offset = Policy::LEN + current_identities_count * BYTES_PER_PUBKEY;

    unsafe {
        sol_memcpy(
            &mut data[new_identity_offset..],
            &identity,
            BYTES_PER_PUBKEY,
        )
    };

    let updated_identities_count = (current_identities_count as u32 + 1).to_le_bytes();
    unsafe {
        sol_memcpy(
            &mut data[3..7],
            &updated_identities_count,
            updated_identities_count.len(),
        )
    };

    Ok(())
}

fn remove_identity(accounts: &[AccountInfo], index: usize) -> ProgramResult {
    let mint = &accounts[0];
    let token_account = &accounts[1];
    let policy = &accounts[2];
    let payer = &accounts[3];
    let owner = &accounts[4];
    let system_program = &accounts[5];

    let mut policy_data = policy.try_borrow_mut_data()?;
    let meta = &policy_data[..Policy::LEN];

    let record = unsafe { Policy::from_bytes(meta) };

    let bump = assert_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key()],
    )?;

    assert_condition(bump == record.nonce, "Policy nonce mismatch")?;
    assert_same_pubkeys("system_program", system_program, &pinocchio_system::ID)?;
    assert_signer("payer", payer)?;
    assert_signer("owner", owner)?;
    assert_writable("payer", payer)?;
    assert_writable("policy", policy)?;
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

    assert_positive_amount("token_account", &account)?;
    assert_ata("token_account", token_account, &owner.key(), &mint.key())?;
    assert_token_owner("token_account", &owner.key(), &account)?;
    assert_mint_association("token_account", &mint.key(), &account)?;

    let position = Policy::LEN + index * BYTES_PER_PUBKEY;

    unsafe {
        sol_memcpy(
            &mut policy_data[position..position + BYTES_PER_PUBKEY],
            &[0u8; BYTES_PER_PUBKEY],
            BYTES_PER_PUBKEY,
        );
    }

    Ok(())
}

fn replace_identity(accounts: &[AccountInfo], index: usize, identity: Pubkey) -> ProgramResult {
    let mint = &accounts[0];
    let token_account = &accounts[1];
    let policy = &accounts[2];
    let payer = &accounts[3];
    let owner = &accounts[4];
    let system_program = &accounts[5];

    let mut policy_data = policy.try_borrow_mut_data()?;
    let meta = &policy_data[..Policy::LEN];

    let record = unsafe { Policy::from_bytes(meta) };

    let bump = assert_pda(
        "policy",
        policy,
        &crate::ID,
        &[b"shield", b"policy", mint.key()],
    )?;

    assert_condition(bump == record.nonce, "Policy nonce mismatch")?;
    assert_same_pubkeys("system_program", system_program, &pinocchio_system::ID)?;
    assert_signer("payer", payer)?;
    assert_signer("owner", owner)?;
    assert_writable("payer", payer)?;
    assert_writable("policy", policy)?;
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

    assert_positive_amount("token_account", &account)?;
    assert_ata("token_account", token_account, &owner.key(), &mint.key())?;
    assert_token_owner("token_account", &owner.key(), &account)?;
    assert_mint_association("token_account", &mint.key(), &account)?;

    let position = Policy::LEN + index * BYTES_PER_PUBKEY;

    unsafe {
        sol_memcpy(
            &mut policy_data[position..position + BYTES_PER_PUBKEY],
            identity.as_ref(),
            BYTES_PER_PUBKEY,
        );
    }

    Ok(())
}

fn close_policy(accounts: &[AccountInfo]) -> ProgramResult {
    let mint = &accounts[0];
    let token_account = &accounts[1];
    let policy = &accounts[2];
    let payer = &accounts[3];
    let owner = &accounts[4];
    let system_program = &accounts[5];

    assert_same_pubkeys("system_program", system_program, &pinocchio_system::ID)?;
    assert_signer("payer", payer)?;
    assert_signer("owner", owner)?;
    assert_writable("payer", payer)?;
    assert_writable("policy", policy)?;
    assert_program_owner("mint", mint, &spl_token_2022::ID.to_bytes())?;
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

    assert_positive_amount("token_account", &account)?;
    assert_ata("token_account", token_account, &owner.key(), &mint.key())?;
    assert_token_owner("token_account", &owner.key(), &account)?;
    assert_mint_association("token_account", &mint.key(), &account)?;

    close_account(policy, payer)?;

    Ok(())
}
