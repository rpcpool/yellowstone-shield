use borsh::{object_length, BorshDeserialize};
use borsh_derive::{BorshDeserialize as DeBorshDeserialize, BorshSerialize as DeBorshSerialize};
use pinocchio::{
    log::sol_log_64,
    msg,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::error::ConfigErrors;

pub const ZEROED: [u8; 32] = [0u8; 32];

#[derive(DeBorshSerialize, DeBorshDeserialize, Default, Debug, Clone)]
pub enum EnumListState {
    #[default]
    Uninitialized,
    ListStateV1(MetaList),
}

#[derive(DeBorshSerialize, DeBorshDeserialize, Default, Debug, Clone)]
pub struct MetaList {
    pub acl_type: AclType,
    pub authority: Option<Pubkey>,
    pub list_items: usize,
}

impl MetaList {
    pub fn get_size(&self) -> Result<usize, ProgramError> {
        object_length(&self).map_err(|_err| ConfigErrors::ErrorGetStructSize.into())
    }

    pub fn get_data_size(&self) -> Result<usize, ProgramError> {
        // Base size + list items size
        Ok(self.get_size()? + (self.list_items * PUBKEY_BYTES))
    }
}

impl EnumListState {
    pub fn get_size(&self) -> Result<usize, ProgramError> {
        object_length(&self).map_err(|_err| ConfigErrors::ErrorGetStructSize.into())
    }
}

#[derive(
    DeBorshSerialize, DeBorshDeserialize, Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord,
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
        msg!("Deserializing ListState");
        msg!("Data length");
        sol_log_64(0, 0, 0, 0, data.len() as u64);

        if data.is_empty() {
            msg!("Data is empty");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut data_mut = data;
        let state = match EnumListState::try_from_slice(&mut data_mut) {
            Ok(s) => {
                msg!("Successfully deserialized EnumListState");
                s
            }
            Err(_) => {
                msg!("Failed to deserialize EnumListState");
                return Err(ProgramError::InvalidAccountData);
            }
        };

        let meta = match &state {
            EnumListState::ListStateV1(meta) => {
                msg!("Got ListStateV1");
                msg!("List items");
                sol_log_64(0, 0, 0, 0, meta.list_items as u64);
                meta.clone()
            }
            EnumListState::Uninitialized => {
                msg!("Account is uninitialized");
                return Err(ProgramError::UninitializedAccount);
            }
        };

        let header_size = state.get_size()?;
        msg!("Header size");
        sol_log_64(0, 0, 0, 0, header_size as u64);
        msg!("Total data length");
        sol_log_64(0, 0, 0, 0, data.len() as u64);
        if data.len() < header_size {
            return Err(ProgramError::InvalidAccountData);
        }

        let raw_addresses_data = &data[header_size..];
        let addresses: &[Pubkey] = bytemuck::try_cast_slice(raw_addresses_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(Self {
            meta,
            list: addresses,
        })
    }
}
