use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use spl_token_2022::state::Account;

use crate::assertions::{
    assert_empty, assert_mint_association, assert_pda, assert_positive_amount, assert_signer,
    assert_token_owner, assert_writable,
};
use crate::instruction::accounts::CreatePolicyAccounts;
use crate::instruction::BlockListInstruction;
use crate::state::{PermissionStrategy, Policy};
use crate::state::{Save, TrySize};
use crate::utils::create_account;

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
            msg!("Instruction: Create");
            create_policy(accounts, strategy, validator_identities)
        }
        _ => {
            msg!("Instruction not recognized");

            Ok(())
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
