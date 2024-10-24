// use std::{collections::HashSet, default};

use borsh::{object_length, BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, program_pack::IsInitialized, pubkey::Pubkey};

use crate::error::ConfigErrors;

// use solana_config_program::ConfigState;

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub enum AclType {
    #[default]
    Deny,
    Allow,
}

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct ConfigListState {
    pub is_initialized: bool,
    pub acl_type: AclType,
    pub authority: Option<Pubkey>,
    pub blocklists: Vec<Pubkey>,
}

impl ConfigListState {
    pub fn max_space() -> u64 {
        std::mem::size_of::<ConfigListState>() as _
    }

    pub fn get_size(&self) -> Result<usize, ProgramError> {
        object_length(&self).map_err(|_err| ConfigErrors::ErrorGetStructSize.into())
    }
}

impl IsInitialized for ConfigListState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
