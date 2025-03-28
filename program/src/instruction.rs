use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};
use solana_program::pubkey::Pubkey;

use crate::state::PermissionStrategy;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum BlockListInstruction {
    /// Creates a shield policy account and a mint account linked to the policy.
    /// The owner of the token extension asset has authority over the policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, name="token_program", desc = "The token program")]
    CreatePolicy {
        strategy: PermissionStrategy,
        identities: Vec<Pubkey>,
    },
    /// Add a new identity to the shield policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    AddIdentity {
        identity: Pubkey,
    },
    /// Remove a identity from the shield policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    RemoveIdentity {
        identity: Pubkey,
    },
}
