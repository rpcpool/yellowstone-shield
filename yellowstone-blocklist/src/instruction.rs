use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::state::AclType;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum ConfigInstructions {
    // 0 - This will include initialization
    InitializeList(InitializeListPayload),
    // 1 - This will update blocklist and rent if is required
    Add(AddListPayload),
    // 2
    RemoveItemList(DeleteListPayload),
    // 3 - Close account and transfer sol to desired account
    CloseAccount,
    // 4 - Update account list type
    UpdateAclType(AclPayload),
    // 5 - Freeze account
    FreezeAccount,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct InitializeListPayload {
    pub acl_type: AclType,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct AclPayload {
    pub acl_type: AclType,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct UpdateAuthPayload {
    pub authority: Option<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct ExtendListPayload {
    pub list: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct AddListPayload {
    pub list: Vec<IndexPubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Clone, Debug)]
pub struct IndexPubkey {
    pub index: u64,
    pub key: Pubkey,
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug)]
pub struct DeleteListPayload {
    pub vec_index: Vec<usize>,
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
                let payload = AddListPayload::try_from_slice(raw)?;
                Self::Add(payload)
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

            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
