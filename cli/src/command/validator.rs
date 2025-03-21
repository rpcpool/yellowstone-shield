use super::RunCommand;
use crate::CommandContext;
use anyhow::Result;
use borsh::BorshDeserialize;
use log::info;
use solana_sdk::signature::Signer;
use solana_sdk::{account::WritableAccount, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use yellowstone_blocklist_client::accounts::Policy;
use yellowstone_blocklist_client::{
    instructions::{AddIdentityBuilder, RemoveIdentityBuilder},
    TransactionBuilder,
};

/// Builder for adding a validator identity to a policy
pub struct AddCommandBuilder<'a> {
    policy: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    validator_identity: Option<&'a Pubkey>,
}

impl<'a> AddCommandBuilder<'a> {
    /// Create a new AddCommandBuilder
    pub fn new() -> Self {
        Self {
            policy: None,
            mint: None,
            validator_identity: None,
        }
    }

    /// Set the policy address
    pub fn policy(mut self, policy: &'a Pubkey) -> Self {
        self.policy = Some(policy);
        self
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
    async fn run(&self, context: CommandContext) -> Result<()> {
        let CommandContext { keypair, client } = context;

        let policy = self.policy.expect("policy must be set");
        let mint = self.mint.expect("mint must be set");
        let validator_identity = self
            .validator_identity
            .expect("validator identity must be set");
        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let add_identity_ix = AddIdentityBuilder::new()
            .policy(policy.clone())
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
            .send_and_confirm_transaction_with_spinner(&tx)
            .await?;

        let mut account_data = client.get_account(policy).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy_data = Policy::deserialize(&mut account_data)?;

        info!("🎉 Validator identity added successfully! 🎉");
        info!("--------------------------------");
        info!("📜 Updated Policy Details:");
        info!("  📬 Policy Address: {}", policy);
        info!("  🪙 Mint Address: {}", mint);
        info!("  📈 Strategy: {:?}", policy_data.strategy);
        info!(
            "  🛡️ Validator Identities: {:?}",
            policy_data.validator_identities
        );
        info!("--------------------------------");

        Ok(())
    }
}

/// Builder for removing a validator identity from a policy
pub struct RemoveCommandBuilder<'a> {
    policy: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    validator_identity: Option<&'a Pubkey>,
}

impl<'a> RemoveCommandBuilder<'a> {
    /// Create a new RemoveCommandBuilder
    pub fn new() -> Self {
        Self {
            policy: None,
            mint: None,
            validator_identity: None,
        }
    }

    /// Set the policy address
    pub fn policy(mut self, policy: &'a Pubkey) -> Self {
        self.policy = Some(policy);
        self
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
    async fn run(&self, context: CommandContext) -> Result<()> {
        let CommandContext { keypair, client } = context;

        let policy = self.policy.expect("policy must be set");
        let mint = self.mint.expect("mint must be set");
        let validator_identity = self
            .validator_identity
            .expect("validator identity must be set");

        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let remove_identity_ix = RemoveIdentityBuilder::new()
            .policy(*policy)
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
            .send_and_confirm_transaction_with_spinner(&tx)
            .await?;

        let mut account_data = client.get_account(policy).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy_data = Policy::deserialize(&mut account_data)?;

        info!("🎉 Validator identity removed successfully! 🎉");
        info!("--------------------------------");
        info!("📜 Updated Policy Details:");
        info!("  📬 Policy Address: {}", policy);
        info!("  🪙 Mint Address: {}", mint);
        info!("  📈 Strategy: {:?}", policy_data.strategy);
        info!(
            "  🛡️ Validator Identities: {:?}",
            policy_data.validator_identities
        );
        info!("--------------------------------");

        Ok(())
    }
}
