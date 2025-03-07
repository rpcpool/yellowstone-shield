use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

pub const ZEROED: [u8; 32] = [0u8; 32];

#[derive(BorshSerialize, BorshDeserialize, Default, Debug, Clone)]
pub enum EnumListState {
    #[default]
    Uninitialized,
    ListStateV1(MetaList),
}

#[derive(BorshDeserialize, BorshSerialize, Default, Debug, Clone)]
pub struct MetaList {
    pub acl_type: AclType,
    pub authority: Option<Pubkey>,
    pub list_items: usize,
}

#[derive(
    BorshDeserialize, BorshSerialize, Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum AclType {
    #[default]
    Deny,
    Allow,
}

#[derive(Default, Debug)]
pub struct ListState {
    pub meta: MetaList,
    pub list: Vec<Pubkey>,
}
