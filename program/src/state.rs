use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};
use shank::ShankAccount;

#[repr(u8)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub enum Kind {
    Policy,
}

#[repr(u8)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Default)]
pub enum PermissionStrategy {
    #[default]
    Deny,
    Allow,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShankAccount)]
pub struct Policy {
    pub kind: u8,
    pub strategy: u8,
    pub nonce: u8,
    pub identities_len: [u8; 4],
}

pub trait Size {
    const LEN: usize;
}

impl Size for Policy {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl Policy {
    pub fn identities_len(&self) -> usize {
        u32::from_le_bytes(self.identities_len) as usize
    }
}

pub trait ZeroCopyLoad {
    unsafe fn from_bytes(bytes: &[u8]) -> &Self;
    unsafe fn load(account_info: &AccountInfo) -> Result<&Self, ProgramError>;
}

impl ZeroCopyLoad for Policy {
    #[inline(always)]
    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Self)
    }

    unsafe fn load(account_info: &AccountInfo) -> Result<&Self, ProgramError> {
        let data = account_info.borrow_data_unchecked();
        Ok(Self::from_bytes(&data[..Self::LEN]))
    }
}
