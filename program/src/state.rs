use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use shank::ShankAccount;

use crate::{error::ShieldError, BYTES_PER_PUBKEY};

pub trait ZeroCopyLoad: Size + Pod {
    #[inline(always)]
    /// Return the State from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of the State.
    unsafe fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        bytemuck::try_from_bytes::<Self>(bytes).map_err(|_| ProgramError::InvalidAccountData)
    }

    /// Return the State from the given account_info.
    ///
    /// # Safety
    ///
    /// The caller must ensure that account_info contains data which is a valid representation of the State.
    unsafe fn load(account_info: &AccountInfo) -> Result<&Self, ProgramError> {
        let data = account_info.borrow_data_unchecked();
        Self::from_bytes(&data[..Self::LEN])
    }
}
pub trait Size {
    const LEN: usize;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, BorshDeserialize, BorshSerialize)]
pub enum Kind {
    Policy,
    PolicyV2,
}

impl TryFrom<u8> for Kind {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, ProgramError> {
        match value {
            0 => Ok(Self::Policy),
            1 => Ok(Self::PolicyV2),
            _ => Err(ShieldError::InvalidPolicyKind.into()),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Default)]
pub enum PermissionStrategy {
    #[default]
    Deny,
    Allow,
}

pub const IDENTITIES_LEN_SIZE: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShankAccount)]
pub struct Policy {
    pub kind: u8,
    pub strategy: u8,
    pub nonce: u8,
    pub identities_len: [u8; 4],
}

impl Policy {
    pub const IDENTITIES_BUFFER_OFFSET: usize = 3;

    pub fn current_identities_len(&self) -> usize {
        u32::from_le_bytes(self.identities_len) as usize
    }

    pub fn identities_len_from_buffer(acc_data_len: usize) -> usize {
        if acc_data_len > Policy::LEN && (acc_data_len - Policy::LEN) % BYTES_PER_PUBKEY == 0 {
            (acc_data_len - Policy::LEN) / BYTES_PER_PUBKEY
        } else {
            0
        }
    }
}

impl Size for Policy {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl ZeroCopyLoad for Policy {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShankAccount)]
pub struct PolicyV2 {
    pub kind: u8,
    pub strategy: u8,
    pub nonce: u8,
    pub mint: Pubkey,
    pub identities_len: [u8; 4],
}

impl PolicyV2 {
    pub const IDENTITIES_BUFFER_OFFSET: usize = 3 + BYTES_PER_PUBKEY;

    pub fn current_identities_len(&self) -> usize {
        u32::from_le_bytes(self.identities_len) as usize
    }

    pub fn identities_len_from_buffer(acc_data_len: usize) -> usize {
        if acc_data_len > PolicyV2::LEN && (acc_data_len - PolicyV2::LEN) % BYTES_PER_PUBKEY == 0 {
            (acc_data_len - PolicyV2::LEN) / BYTES_PER_PUBKEY
        } else {
            0
        }
    }
}

impl Size for PolicyV2 {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl ZeroCopyLoad for PolicyV2 {}
