use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use yellowstone_shield_client::instructions::{
    AddIdentity as AddIdentityIxAccounts, AddIdentityInstructionArgs as AddIdentityIxData,
    CreatePolicy as CreatePolicyIxAccounts, CreatePolicyInstructionArgs as CreatePolicyIxData,
    RemoveIdentity as RemoveIdentityIxAccounts,
    RemoveIdentityInstructionArgs as RemoveIdentityIxData,
};

use yellowstone_shield_client::ID;

/// Shield Instructions
#[derive(Debug)]
#[allow(dead_code)]
pub enum ShieldProgramIx {
    CreatePolicy(CreatePolicyIxAccounts, CreatePolicyIxData),
    AddIdentity(AddIdentityIxAccounts, AddIdentityIxData),
    RemoveIdentity(RemoveIdentityIxAccounts, RemoveIdentityIxData),
}

pub fn parse_instruction(
    program_id: &Pubkey,
    accounts: &[Pubkey],
    data: &[u8],
) -> Result<ShieldProgramIx, String> {
    if program_id != &ID {
        return Err(format!(
            "Invalid program ID: expected {}, got {}",
            ID, program_id
        ));
    }

    if data.is_empty() {
        return Err("Instruction data is empty".to_owned());
    }

    let accounts_len = accounts.len();
    let ix_discriminator = data[0];
    let mut ix_data = &data[1..];

    match ix_discriminator {
        0 => {
            check_min_accounts_req(accounts_len, 6)?;
            let ix_accounts = CreatePolicyIxAccounts {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                payer: accounts[3],
                owner: accounts[4],
                system_program: accounts[5],
            };
            let de_ix_data: CreatePolicyIxData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| e.to_string())?;
            Ok(ShieldProgramIx::CreatePolicy(ix_accounts, de_ix_data))
        }
        1 => {
            check_min_accounts_req(accounts_len, 6)?;
            let ix_accounts = AddIdentityIxAccounts {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                payer: accounts[3],
                owner: accounts[4],
                system_program: accounts[5],
            };
            let de_ix_data: AddIdentityIxData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| e.to_string())?;
            Ok(ShieldProgramIx::AddIdentity(ix_accounts, de_ix_data))
        }
        2 => {
            check_min_accounts_req(accounts_len, 4)?;
            let ix_accounts = RemoveIdentityIxAccounts {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                owner: accounts[3],
            };
            let de_ix_data: RemoveIdentityIxData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| e.to_string())?;
            Ok(ShieldProgramIx::RemoveIdentity(ix_accounts, de_ix_data))
        }
        _ => Err("Invalid Instruction discriminator".to_owned()),
    }
}

fn check_min_accounts_req(actual: usize, expected: usize) -> Result<(), String> {
    if actual < expected {
        Err(format!(
            "Too few accounts provided: expected {expected}, got {actual}"
        ))
    } else {
        Ok(())
    }
}
