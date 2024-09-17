use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
// use solana_config_program::ConfigState;


#[derive(BorshSerialize, BorshDeserialize, Serialize, Default)]
pub struct ConfigListState {
    pub blocklists: HashMap<String, Vec<String>>
}

impl ConfigListState {
    pub fn max_space() -> u64 {
        10000
    }
}