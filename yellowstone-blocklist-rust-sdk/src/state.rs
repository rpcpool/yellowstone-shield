use crate::error::{BlocklistError, BlocklistResult};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{borsh1::try_from_slice_unchecked, pubkey::Pubkey};

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
    pub fn get_size(&self) -> BlocklistResult<usize> {
        borsh::object_length(&self).map_err(|e| {
            BlocklistError::SerializationError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            ))
        })
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

#[derive(Debug)]
pub struct ListState {
    pub meta: MetaList,
    pub list: Vec<Pubkey>,
}

impl ListState {
    pub fn deserialize(data: &[u8]) -> BlocklistResult<ListState> {
        let state = try_from_slice_unchecked::<EnumListState>(data).map_err(|e| {
            BlocklistError::SerializationError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            ))
        })?;

        let meta = match state.clone() {
            EnumListState::ListStateV1(meta) => Ok(meta),
            EnumListState::Uninitialized => Err(BlocklistError::InvalidState(
                "Account is uninitialized".to_string(),
            )),
        }?;

        let raw_addresses_data = data
            .get(state.get_size()?..)
            .ok_or_else(|| BlocklistError::InvalidState("Invalid data length".to_string()))?;

        let addresses: &[Pubkey] = bytemuck::try_cast_slice(raw_addresses_data)
            .map_err(|_| BlocklistError::InvalidState("Invalid pubkey data".to_string()))?;

        Ok(Self {
            meta,
            list: addresses.to_vec(),
        })
    }
}
