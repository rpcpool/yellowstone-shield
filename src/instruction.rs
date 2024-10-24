use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::state::AclType;

// use solana_config_program::ConfigState;

pub enum ConfigInstructions {
    // 0 - This will include initialization
    AddBlockList {
        acl_type: AclType,
        blocklist: Vec<Pubkey>,
    },
    // 1 - This will update blocklist and rent if is required
    UpdateBlocklist {
        edit_list: Vec<IndexPubkey>,
    },
    // 2
    CloseAccount,
    // 3
    UpdateAuthority,
    // 4
    UpdateAclType {
        acl_type: AclType,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct ConfigPayload {
    acl_type: AclType,
    blocklist: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct UpdateAclPayload {
    acl_type: AclType,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct UpdateAuthPayload {
    authority: Option<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct UpdateBlocklistPayload {
    edit_list: Vec<IndexPubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct IndexPubkey {
    pub index: u64,
    pub key: Pubkey,
}

impl ConfigInstructions {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, raw) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => {
                let payload = ConfigPayload::try_from_slice(raw)?;
                Self::AddBlockList {
                    blocklist: payload.blocklist,
                    acl_type: payload.acl_type,
                }
            }
            1 => {
                let payload = UpdateBlocklistPayload::try_from_slice(raw)?;
                Self::UpdateBlocklist {
                    edit_list: payload.edit_list,
                }
            }
            2 => Self::CloseAccount,
            3 => Self::UpdateAuthority,
            4 => {
                let payload = UpdateAclPayload::try_from_slice(raw)?;
                Self::UpdateAclType {
                    acl_type: payload.acl_type,
                }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
