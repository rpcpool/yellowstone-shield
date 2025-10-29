// Re-export from program
pub use yellowstone_shield::state::PermissionStrategy;
use {
    super::accounts::{ParseError, ParseResult, ID},
    borsh::BorshDeserialize,
    solana_program::pubkey::Pubkey,
};

/// Shield instruction data types
#[derive(Debug, BorshDeserialize)]
pub struct CreatePolicyData {
    pub strategy: PermissionStrategy,
}

#[derive(Debug, BorshDeserialize)]
pub struct AddIdentityData {
    pub identity: Pubkey,
}

#[derive(Debug, BorshDeserialize)]
pub struct RemoveIdentityData {
    pub index: usize,
}

#[derive(Debug, BorshDeserialize)]
pub struct ReplaceIdentityData {
    pub index: usize,
    pub identity: Pubkey,
}

/// Shield Instructions
#[derive(Debug)]
#[allow(dead_code)]
pub enum ShieldInstruction {
    CreatePolicy {
        mint: Pubkey,
        token_account: Pubkey,
        policy: Pubkey,
        payer: Pubkey,
        owner: Pubkey,
        system_program: Pubkey,
        data: CreatePolicyData,
    },
    AddIdentity {
        mint: Pubkey,
        token_account: Pubkey,
        policy: Pubkey,
        payer: Pubkey,
        owner: Pubkey,
        system_program: Pubkey,
        data: AddIdentityData,
    },
    RemoveIdentity {
        mint: Pubkey,
        token_account: Pubkey,
        policy: Pubkey,
        owner: Pubkey,
        data: RemoveIdentityData,
    },
    ReplaceIdentity {
        mint: Pubkey,
        token_account: Pubkey,
        policy: Pubkey,
        owner: Pubkey,
        data: ReplaceIdentityData,
    },
    ClosePolicy {
        mint: Pubkey,
        token_account: Pubkey,
        policy: Pubkey,
        payer: Pubkey,
        owner: Pubkey,
        system_program: Pubkey,
    },
}

/// Parse a Shield program instruction from raw instruction data
pub fn parse_instruction(
    program_id: &Pubkey,
    accounts: &[Pubkey],
    data: &[u8],
) -> ParseResult<ShieldInstruction> {
    // Verify program ID
    if program_id != &ID {
        return Err(ParseError::Custom(format!(
            "Invalid program ID: expected {}, got {}",
            ID, program_id
        )));
    }

    if data.is_empty() {
        return Err(ParseError::from("Instruction data is empty"));
    }

    let accounts_len = accounts.len();
    let ix_discriminator = data[0];
    let mut ix_data = &data[1..];

    match ix_discriminator {
        0 => {
            check_min_accounts_req(accounts_len, 6)?;
            let data: CreatePolicyData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| {
                    ParseError::Custom(format!("Failed to deserialize CreatePolicy: {}", e))
                })?;
            Ok(ShieldInstruction::CreatePolicy {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                payer: accounts[3],
                owner: accounts[4],
                system_program: accounts[5],
                data,
            })
        }
        1 => {
            check_min_accounts_req(accounts_len, 6)?;
            let data: AddIdentityData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| {
                    ParseError::Custom(format!("Failed to deserialize AddIdentity: {}", e))
                })?;
            Ok(ShieldInstruction::AddIdentity {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                payer: accounts[3],
                owner: accounts[4],
                system_program: accounts[5],
                data,
            })
        }
        2 => {
            check_min_accounts_req(accounts_len, 4)?;
            let data: RemoveIdentityData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| {
                    ParseError::Custom(format!("Failed to deserialize RemoveIdentity: {}", e))
                })?;
            Ok(ShieldInstruction::RemoveIdentity {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                owner: accounts[3],
                data,
            })
        }
        3 => {
            check_min_accounts_req(accounts_len, 4)?;
            let data: ReplaceIdentityData =
                BorshDeserialize::deserialize(&mut ix_data).map_err(|e| {
                    ParseError::Custom(format!("Failed to deserialize ReplaceIdentity: {}", e))
                })?;
            Ok(ShieldInstruction::ReplaceIdentity {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                owner: accounts[3],
                data,
            })
        }
        4 => {
            check_min_accounts_req(accounts_len, 6)?;
            Ok(ShieldInstruction::ClosePolicy {
                mint: accounts[0],
                token_account: accounts[1],
                policy: accounts[2],
                payer: accounts[3],
                owner: accounts[4],
                system_program: accounts[5],
            })
        }
        _ => Err(ParseError::from("Invalid Instruction discriminator")),
    }
}

fn check_min_accounts_req(actual: usize, expected: usize) -> ParseResult<()> {
    if actual < expected {
        Err(ParseError::from(format!(
            "Too few accounts provided: expected {expected}, got {actual}"
        )))
    } else {
        Ok(())
    }
}

/// Get the Shield program ID
pub fn program_id() -> Pubkey {
    ID
}
