pub mod assertions;
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod system;

use pinocchio::pubkey::Pubkey;

pub const BYTES_PER_PUBKEY: usize = core::mem::size_of::<Pubkey>();

pinocchio_pubkey::declare_id!("b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W");
