use super::{CommandComplete, RunCommand, RunResult, SolanaAccount};
use crate::command::CommandContext;
use anyhow::Result;
use borsh::BorshDeserialize;
use log::info;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signer;
use solana_sdk::{account::WritableAccount, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions},
    pod::PodMint,
    state::Mint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as TokenBorshDeserialize, state::TokenMetadata,
};
use yellowstone_shield_client::accounts::Policy;
use yellowstone_shield_client::types::PermissionStrategy;
use yellowstone_shield_client::{
    instructions::{AddIdentityBuilder, RemoveIdentityBuilder},
    TransactionBuilder,
};

/// Builder for adding a validator identity to a policy
pub struct AddCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    validator_identity: Option<&'a Pubkey>,
}

impl<'a> AddCommandBuilder<'a> {
    /// Create a new AddCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            validator_identity: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the validator identity to add
    pub fn validator_identity(mut self, validator_identity: &'a Pubkey) -> Self {
        self.validator_identity = Some(validator_identity);
        self
    }
}

#[async_trait::async_trait]
impl<'a> RunCommand for AddCommandBuilder<'a> {
    /// Execute the addition of a validator identity to the policy
    async fn run(&self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        let (address, _) = Policy::find_pda(mint);
        let validator_identity = self
            .validator_identity
            .expect("validator identity must be set");
        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let add_identity_ix = AddIdentityBuilder::new()
            .policy(address.clone())
            .mint(mint.clone())
            .token_account(token_account)
            .payer(keypair.pubkey())
            .validator_identity(*validator_identity)
            .instruction();

        let last_blockhash = client.get_latest_blockhash().await?;

        let tx = TransactionBuilder::build()
            .instruction(add_identity_ix)
            .signer(&keypair)
            .payer(&keypair.pubkey())
            .recent_blockhash(last_blockhash)
            .transaction();

        client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .await?;

        let mut account_data = client.get_account(&address).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy = Policy::deserialize(&mut account_data)?;

        let mut mint_data = client.get_account(&mint).await?;
        let account_data: &[u8] = mint_data.data_as_mut_slice();

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(&account_data).unwrap();
        let mut mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(&mut mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, token_metadata),
            SolanaAccount(address, policy),
        ))
    }
}

/// Builder for removing a validator identity from a policy
pub struct RemoveCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    validator_identity: Option<&'a Pubkey>,
}

impl<'a> RemoveCommandBuilder<'a> {
    /// Create a new RemoveCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            validator_identity: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the validator identity to remove
    pub fn validator_identity(mut self, validator_identity: &'a Pubkey) -> Self {
        self.validator_identity = Some(validator_identity);
        self
    }
}

#[async_trait::async_trait]
impl<'a> RunCommand for RemoveCommandBuilder<'a> {
    /// Execute the removal of a validator identity from the policy
    async fn run(&self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        let (address, _) = Policy::find_pda(mint);
        let validator_identity = self
            .validator_identity
            .expect("validator identity must be set");

        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let remove_identity_ix = RemoveIdentityBuilder::new()
            .policy(address)
            .mint(*mint)
            .payer(keypair.pubkey())
            .token_account(token_account)
            .validator_identity(*validator_identity)
            .instruction();

        let last_blockhash = client.get_latest_blockhash().await?;

        let tx = TransactionBuilder::build()
            .instruction(remove_identity_ix)
            .signer(&keypair)
            .payer(&keypair.pubkey())
            .recent_blockhash(last_blockhash)
            .transaction();

        client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .await?;

        let mut account_data = client.get_account(&address).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy = Policy::deserialize(&mut account_data)?;

        let mut mint_data = client.get_account(&mint).await?;
        let account_data: &[u8] = mint_data.data_as_mut_slice();

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(&account_data).unwrap();
        let mut mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(&mut mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, token_metadata),
            SolanaAccount(address, policy),
        ))
    }
}
