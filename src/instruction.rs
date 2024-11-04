use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::state::AclType;

#[derive(BorshDeserialize, BorshSerialize)]
pub enum ConfigInstructions {
    // 0 - This will include initialization
    InitializeList(InitializeListPayload),
    // 1 - This will update blocklist and rent if is required
    ExtendList(ExtendListPayload),
    // 2
    RemoveItemList(DeleteListPayload),
    // 3 - Close account and transfer sol to desired account
    CloseAccount,
    // 4 - Update account list type
    UpdateAclType(AclPayload),
    // 5 - Freeze account
    FreezeAccount,
    // 6 - Update item list
    UpdateList(EditListPayload),
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct InitializeListPayload {
    pub acl_type: AclType,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct AclPayload {
    pub acl_type: AclType,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct UpdateAuthPayload {
    pub authority: Option<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct ExtendListPayload {
    pub list: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct EditListPayload {
    pub list: Vec<IndexPubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Clone)]
pub struct IndexPubkey {
    pub index: u64,
    pub key: Pubkey,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct DeleteListPayload {
    pub index: usize,
}

impl ConfigInstructions {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, raw) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => {
                let payload = InitializeListPayload::try_from_slice(raw)?;

                Self::InitializeList(payload)
            }
            1 => {
                let payload = ExtendListPayload::try_from_slice(raw)?;
                Self::ExtendList(payload)
            }
            2 => {
                let payload = DeleteListPayload::try_from_slice(raw)?;
                Self::RemoveItemList(payload)
            }
            3 => Self::CloseAccount,
            4 => {
                let payload = AclPayload::try_from_slice(raw)?;
                Self::UpdateAclType(payload)
            }
            5 => Self::FreezeAccount,
            6 => {
                let payload = EditListPayload::try_from_slice(raw)?;
                Self::UpdateList(payload)
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
