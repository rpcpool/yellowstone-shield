use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;
use shank::ShankInstruction;

use crate::state::PermissionStrategy;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankInstruction)]
#[rustfmt::skip]
pub enum ShieldInstruction {
    /// Creates a shield policy account and a mint account linked to the policy.
    /// The owner of the token extension asset has authority over the policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, writable, signer, name="owner", desc = "The owner of the token account")]
    #[account(5, name="system_program", desc = "The system program")]
    CreatePolicy {
        strategy: PermissionStrategy,
    },
    /// Add a new identity to the shield policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, writable, signer, name="owner", desc = "The owner of the token account")]
    #[account(5, name="system_program", desc = "The system program")]
    AddIdentity {
        identity: Pubkey,
    },
    /// Remove a identity from the shield policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="owner", desc = "The owner of the token account")]
    RemoveIdentity {
        index: usize,
    },
    /// Replace an identity by its index for the shield policy.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="owner", desc = "The owner of the token account")]
    ReplaceIdentity {
        index: usize,
        identity: Pubkey,
    },
    /// Close the shield policy account.
    #[account(0, name="mint", desc = "The token extensions mint account linked to the policy")]
    #[account(1, name="token_account", desc = "The authority over the policy based on token ownership of the mint")]
    #[account(2, writable, name="policy", desc = "The shield policy account")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, writable, signer, name="owner", desc = "The owner of the token account")]
    #[account(5, name="system_program", desc = "The system program")]
    ClosePolicy
}
