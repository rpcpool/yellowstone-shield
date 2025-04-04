use borsh::{object_length, to_writer, BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::slot_history::AccountInfo;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub enum Kind {
    Policy,
}
pub trait Save: BorshSerialize {
    fn save(&self, account: &AccountInfo) -> ProgramResult {
        to_writer(&mut account.data.borrow_mut()[..], self).map_err(Into::into)
    }
}

pub trait Load: BorshDeserialize {
    fn load(account: &AccountInfo) -> Result<Self, ProgramError>
    where
        Self: Sized,
    {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Self::deserialize(&mut bytes).map_err(Into::into)
    }
}

pub trait TrySize: BorshSerialize {
    fn try_size(&self) -> Result<usize, ProgramError> {
        object_length(&self).map_err(Into::into)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub enum PermissionStrategy {
    Allow,
    Deny,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct Policy {
    pub kind: Kind,
    pub strategy: PermissionStrategy,
    pub nonce: u8,
    pub identities: Vec<Pubkey>,
}

impl Policy {
    pub fn new(nonce: u8, strategy: PermissionStrategy, identities: Vec<Pubkey>) -> Self {
        Self {
            kind: Kind::Policy,
            strategy,
            nonce,
            identities,
        }
    }

    pub fn seeds(mint_key: &Pubkey) -> Vec<&[u8]> {
        vec![b"shield", b"policy", mint_key.as_ref()]
    }

    pub fn find_policy_program_address(mint: &Pubkey, nonce: Option<u8>) -> (Pubkey, u8) {
        let mut seeds = Self::seeds(mint);
        let slice;

        if let Some(n) = nonce {
            slice = [n];
            seeds.push(&slice);
        }

        Pubkey::find_program_address(&seeds, &crate::ID)
    }
}

impl TrySize for Policy {}
impl Save for Policy {}
impl Load for Policy {}
