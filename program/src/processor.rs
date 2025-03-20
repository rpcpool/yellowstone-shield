use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use spl_token_2022::state::Account;

use crate::assertions::{
    assert_empty, assert_mint_association, assert_pda, assert_positive_amount,
    assert_program_owner, assert_signer, assert_token_owner, assert_writable,
};
use crate::instruction::accounts::{
    AddIdentityAccounts, CreatePolicyAccounts, RemoveIdentityAccounts,
};
use crate::instruction::BlockListInstruction;
use crate::state::{Load, Save, TrySize};
use crate::state::{PermissionStrategy, Policy};
use crate::utils::{create_account, realloc_account};

pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: BlockListInstruction = BlockListInstruction::try_from_slice(instruction_data)?;
    match instruction {
        BlockListInstruction::CreatePolicy {
            strategy,
            validator_identities,
        } => {
            msg!("Instruction: Create Policy");
            create_policy(accounts, strategy, validator_identities)
        }
        BlockListInstruction::AddIdentity { validator_identity } => {
            msg!("Instruction: Add Identity");
            add_identity(accounts, validator_identity)
        }
        BlockListInstruction::RemoveIdentity { validator_identity } => {
            msg!("Instruction: Remove Identity");
            remove_identity(accounts, validator_identity)
        }
    }
}

fn create_policy<'a>(
    accounts: &'a [AccountInfo<'a>],
    strategy: PermissionStrategy,
    validator_identities: Vec<Pubkey>,
) -> ProgramResult {
    let ctx = CreatePolicyAccounts::context(accounts)?;

    let policy_bump = assert_pda(
        "policy",
        ctx.accounts.policy,
        &crate::ID,
        &Policy::seeds(ctx.accounts.mint.key),
    )?;
    assert_signer("payer", ctx.accounts.payer)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_writable("policy", ctx.accounts.policy)?;
    assert_program_owner("mint", ctx.accounts.mint, &spl_token_2022::id())?;
    assert_program_owner(
        "token_account",
        ctx.accounts.token_account,
        &spl_token_2022::id(),
    )?;

    let token_account_data = &ctx.accounts.token_account.try_borrow_data()?;
    let token_account = Account::unpack(token_account_data)?;

    assert_positive_amount("token_account", &token_account)?;
    assert_token_owner("token_account", ctx.accounts.payer.key, &token_account)?;
    assert_mint_association("token_account", ctx.accounts.mint.key, &token_account)?;
    assert_empty("policy", ctx.accounts.policy)?;

    let policy = Policy::new(strategy, validator_identities);

    let mut seeds = Policy::seeds(ctx.accounts.mint.key);
    let bump = [policy_bump];
    seeds.push(&bump);

    create_account(
        ctx.accounts.policy,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        policy.try_size()?,
        &crate::ID,
        Some(&[&seeds]),
    )?;

    policy.save(ctx.accounts.policy)
}

fn add_identity<'a>(accounts: &'a [AccountInfo<'a>], validator_identity: Pubkey) -> ProgramResult {
    let ctx = AddIdentityAccounts::context(accounts)?;

    assert_pda(
        "policy",
        ctx.accounts.policy,
        &crate::ID,
        &Policy::seeds(ctx.accounts.mint.key),
    )?;
    assert_signer("payer", ctx.accounts.payer)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_writable("policy", ctx.accounts.policy)?;
    assert_program_owner("mint", ctx.accounts.mint, &spl_token_2022::id())?;
    assert_program_owner(
        "token_account",
        ctx.accounts.token_account,
        &spl_token_2022::id(),
    )?;

    let token_account_data = &ctx.accounts.token_account.try_borrow_data()?;
    let token_account = Account::unpack(token_account_data)?;

    assert_positive_amount("token_account", &token_account)?;
    assert_token_owner("token_account", ctx.accounts.payer.key, &token_account)?;
    assert_mint_association("token_account", ctx.accounts.mint.key, &token_account)?;

    let mut policy: Policy = Policy::load(ctx.accounts.policy)?;
    policy.validator_identities.push(validator_identity);

    realloc_account(
        ctx.accounts.policy,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        policy.try_size()?,
        false,
    )?;

    policy.save(ctx.accounts.policy)
}

fn remove_identity<'a>(
    accounts: &'a [AccountInfo<'a>],
    validator_identity: Pubkey,
) -> ProgramResult {
    let ctx = RemoveIdentityAccounts::context(accounts)?;

    assert_pda(
        "policy",
        ctx.accounts.policy,
        &crate::ID,
        &Policy::seeds(ctx.accounts.mint.key),
    )?;
    assert_signer("payer", ctx.accounts.payer)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_writable("policy", ctx.accounts.policy)?;
    assert_program_owner("mint", ctx.accounts.mint, &spl_token_2022::id())?;
    assert_program_owner(
        "token_account",
        ctx.accounts.token_account,
        &spl_token_2022::id(),
    )?;

    let token_account_data = &ctx.accounts.token_account.try_borrow_data()?;
    let token_account = Account::unpack(token_account_data)?;

    assert_positive_amount("token_account", &token_account)?;
    assert_token_owner("token_account", ctx.accounts.payer.key, &token_account)?;
    assert_mint_association("token_account", ctx.accounts.mint.key, &token_account)?;

    let mut policy: Policy = Policy::load(ctx.accounts.policy)?;
    if let Some(pos) = policy
        .validator_identities
        .iter()
        .position(|id| id == &validator_identity)
    {
        policy.validator_identities.remove(pos);
    }

    realloc_account(
        ctx.accounts.policy,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        policy.try_size()?,
        true,
    )?;

    policy.save(ctx.accounts.policy)
}
