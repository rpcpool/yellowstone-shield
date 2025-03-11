use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

use crate::state::AclType;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum ConfigInstructions {
    // 0 - This will include initialization
    InitializeList(InitializeListPayload),
    // 1 - This will update blocklist and rent if is required
    Add(AddListPayload),
    // 2 - This will remove item from list and transfer sol to desired account
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
