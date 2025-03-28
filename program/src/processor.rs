use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::assertions::{
    assert_ata, assert_empty, assert_mint_association, assert_pda, assert_positive_amount,
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
            identities,
        } => {
            msg!("Instruction: Create Policy");
            create_policy(accounts, strategy, identities)
        }
        BlockListInstruction::AddIdentity { identity } => {
            msg!("Instruction: Add Identity");
            add_identity(accounts, identity)
        }
        BlockListInstruction::RemoveIdentity { identity } => {
            msg!("Instruction: Remove Identity");
            remove_identity(accounts, identity)
        }
    }
}

fn create_policy<'a>(
    accounts: &'a [AccountInfo<'a>],
    strategy: PermissionStrategy,
    identities: Vec<Pubkey>,
) -> ProgramResult {
    let ctx = CreatePolicyAccounts::context(accounts)?;

    let nonce = assert_pda(
        "policy",
        ctx.accounts.policy,
        &crate::ID,
        &Policy::seeds(ctx.accounts.mint.key),
    )?;
    assert_signer("payer", ctx.accounts.payer)?;
    assert_signer("owner", ctx.accounts.owner)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_writable("policy", ctx.accounts.policy)?;
    assert_ata(
        "token_account",
        ctx.accounts.token_account,
        ctx.accounts.owner.key,
        ctx.accounts.mint.key,
    )?;
    assert_program_owner("mint", ctx.accounts.mint, &spl_token_2022::id())?;
    assert_program_owner(
        "token_account",
        ctx.accounts.token_account,
        &spl_token_2022::id(),
    )?;

    let token_account_data = &ctx.accounts.token_account.try_borrow_data()?;
    let token_account = spl_token_2022::extension::StateWithExtensions::<
        spl_token_2022::state::Account,
    >::unpack(token_account_data)?;

    assert_positive_amount("token_account", &token_account)?;
    assert_token_owner("token_account", ctx.accounts.owner.key, &token_account)?;
    assert_mint_association("token_account", ctx.accounts.mint.key, &token_account)?;
    assert_empty("policy", ctx.accounts.policy)?;

    let policy = Policy::new(nonce, strategy, identities);

    let mut seeds = Policy::seeds(ctx.accounts.mint.key);
    let bump = [nonce];
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

fn add_identity<'a>(accounts: &'a [AccountInfo<'a>], identity: Pubkey) -> ProgramResult {
    let ctx = AddIdentityAccounts::context(accounts)?;

    let mut policy: Policy = Policy::load(ctx.accounts.policy)?;

    let bump = assert_pda(
        "policy",
        ctx.accounts.policy,
        &crate::ID,
        &Policy::seeds(ctx.accounts.mint.key),
    )?;

    assert_eq!(bump, policy.nonce);
    assert_signer("payer", ctx.accounts.payer)?;
    assert_signer("owner", ctx.accounts.owner)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_writable("policy", ctx.accounts.policy)?;
    assert_program_owner("mint", ctx.accounts.mint, &spl_token_2022::id())?;
    assert_program_owner(
        "token_account",
        ctx.accounts.token_account,
        &spl_token_2022::id(),
    )?;
    let token_account_data = &ctx.accounts.token_account.try_borrow_data()?;
    let token_account = spl_token_2022::extension::StateWithExtensions::<
        spl_token_2022::state::Account,
    >::unpack(token_account_data)?;

    assert_positive_amount("token_account", &token_account)?;
    assert_ata(
        "token_account",
        ctx.accounts.token_account,
        ctx.accounts.owner.key,
        ctx.accounts.mint.key,
    )?;
    assert_token_owner("token_account", ctx.accounts.owner.key, &token_account)?;
    assert_mint_association("token_account", ctx.accounts.mint.key, &token_account)?;

    policy.identities.push(identity);

    realloc_account(
        ctx.accounts.policy,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        policy.try_size()?,
        false,
    )?;

    policy.save(ctx.accounts.policy)
}

fn remove_identity<'a>(accounts: &'a [AccountInfo<'a>], identity: Pubkey) -> ProgramResult {
    let ctx = RemoveIdentityAccounts::context(accounts)?;

    let mut policy: Policy = Policy::load(ctx.accounts.policy)?;

    let bump = assert_pda(
        "policy",
        ctx.accounts.policy,
        &crate::ID,
        &Policy::seeds(ctx.accounts.mint.key),
    )?;

    assert_eq!(bump, policy.nonce);
    assert_signer("payer", ctx.accounts.payer)?;
    assert_signer("owner", ctx.accounts.owner)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_writable("policy", ctx.accounts.policy)?;
    assert_program_owner("mint", ctx.accounts.mint, &spl_token_2022::id())?;
    assert_program_owner(
        "token_account",
        ctx.accounts.token_account,
        &spl_token_2022::id(),
    )?;

    let token_account_data = &ctx.accounts.token_account.try_borrow_data()?;
    let token_account = spl_token_2022::extension::StateWithExtensions::<
        spl_token_2022::state::Account,
    >::unpack(token_account_data)?;

    assert_positive_amount("token_account", &token_account)?;
    assert_ata(
        "token_account",
        ctx.accounts.token_account,
        ctx.accounts.owner.key,
        ctx.accounts.mint.key,
    )?;
    assert_token_owner("token_account", ctx.accounts.owner.key, &token_account)?;
    assert_mint_association("token_account", ctx.accounts.mint.key, &token_account)?;

    if let Some(pos) = policy.identities.iter().position(|id| id == &identity) {
        policy.identities.remove(pos);
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
