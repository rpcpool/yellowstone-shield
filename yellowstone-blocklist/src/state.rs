use borsh::{object_length, BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh1::try_from_slice_unchecked, program_error::ProgramError, pubkey::Pubkey,
};

use crate::error::ConfigErrors;

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

impl EnumListState {
    pub fn get_size(&self) -> Result<usize, ProgramError> {
        object_length(&self).map_err(|_err| ConfigErrors::ErrorGetStructSize.into())
    }
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
pub struct ListState<'a> {
    pub meta: MetaList,
    pub list: &'a [Pubkey],
}

impl<'a> ListState<'a> {
    pub fn deserialize(data: &'a [u8]) -> Result<ListState<'a>, ProgramError> {
        let state = try_from_slice_unchecked::<EnumListState>(data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let meta = match state.clone() {
            EnumListState::ListStateV1(meta) => Ok(meta),
            EnumListState::Uninitialized => Err(ProgramError::UninitializedAccount),
        }?;

        let raw_addresses_data = data
            .get(state.get_size()?..)
            .ok_or(ProgramError::InvalidAccountData)?;
        let addresses: &[Pubkey] = bytemuck::try_cast_slice(raw_addresses_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(Self {
            meta,
            list: addresses,
        })
    }
}
