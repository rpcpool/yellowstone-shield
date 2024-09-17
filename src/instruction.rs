use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
use solana_program::program_error::ProgramError;
// use solana_config_program::ConfigState;

pub enum ConfigInstructions {
    AddOrEditBlocklist {
        pubkey: String,
        blocklist: Vec<String>,
    },
    DeleteBlocklist {
        pubkey: String,
    },
    InitializeAccount
}

#[derive(BorshDeserialize, BorshSerialize, Default, Serialize)]
pub struct ConfigPayload {
    pubkey: String,
    blocklist: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Serialize)]
pub struct DeleteConfigPayload {
    pubkey: String,
}

impl ConfigInstructions {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, raw) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => {
                let payload = ConfigPayload::try_from_slice(raw)?;
                Self::AddOrEditBlocklist {
                    pubkey: payload.pubkey,
                    blocklist: payload.blocklist,
                }
            }
            1 => {
                let payload = DeleteConfigPayload::try_from_slice(raw)?;
                Self::DeleteBlocklist {
                    pubkey: payload.pubkey,
                }
            }
            2 => {
                Self::InitializeAccount
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

