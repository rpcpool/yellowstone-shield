use anyhow::Result;
use log::info;
use solana_sdk::{account::WritableAccount, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;

use crate::CommandContext;

use super::RunCommand;
use borsh::BorshDeserialize;
use solana_sdk::signature::{Keypair, Signer};
use spl_token_2022::{extension::ExtensionType, state::Mint};
use yellowstone_blocklist_client::{
    accounts::Policy, instructions::CreatePolicyBuilder, types::PermissionStrategy,
    CreateAccountBuilder, CreateAsscoiatedTokenAccountBuilder, InitializeMint2Builder,
    MetadataPointerInitializeBuilder, TokenExtensionsMintToBuilder, TransactionBuilder,
};
/// Builder for creating a new policy
pub struct CreateCommandBuilder<'a> {
    strategy: Option<&'a PermissionStrategy>,
    validator_identities: Option<&'a Vec<Pubkey>>,
}

impl<'a> CreateCommandBuilder<'a> {
    /// Create a new PolicyBuilder
    pub fn new() -> Self {
        Self {
            strategy: None,
            validator_identities: None,
        }
    }

    /// Set the strategy for the policy
    pub fn strategy(mut self, strategy: &'a PermissionStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Add a validator identity to the policy
    pub fn validator_identities(mut self, validator_identities: &'a Vec<Pubkey>) -> Self {
        self.validator_identities = Some(validator_identities);
        self
    }
}

#[async_trait::async_trait]
impl<'a> RunCommand for CreateCommandBuilder<'a> {
    /// Execute the creation of the policy
    async fn run(&self, context: CommandContext) -> Result<()> {
        let CommandContext { keypair, client } = context;

        // Given a PDA derived from the payer's public key.
        let mint = Keypair::new();
        // Create a token account for the payer.
        let payer_token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            &mint.pubkey(),
            &spl_token_2022::ID,
        );
        // Mock the validator identity.
        let validator_identities = self
            .validator_identities
            .expect("validator identities must be set");
        // Calculate the space required for the mint account with extensions.
        let space =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer])
                .unwrap();

        let create_mint_ix = CreateAccountBuilder::build()
            .payer(&keypair.pubkey())
            .account(&mint.pubkey())
            .space(space)
            .owner(&spl_token_2022::id())
            .instruction();
        // Initialize metadata pointer extension.
        let init_metadata_pointer_ix = MetadataPointerInitializeBuilder::build()
            .mint(&mint.pubkey())
            .metadata(mint.pubkey())
            .authority(keypair.pubkey())
            .instruction();

        let init_mint_ix = InitializeMint2Builder::build()
            .mint(&mint.pubkey())
            .mint_authority(&keypair.pubkey())
            .instruction();

        // Create the policy account.
        let (address, _) = Policy::find_pda(&mint.pubkey());
        let create_policy_ix = CreatePolicyBuilder::new()
            .policy(address)
            .mint(mint.pubkey())
            .payer(keypair.pubkey())
            .token_account(payer_token_account)
            .validator_identities(validator_identities.to_vec())
            .strategy(yellowstone_blocklist_client::types::PermissionStrategy::Allow)
            .instruction();

        // Initialize the payer's token account.
        let init_payer_token_account_ix = CreateAsscoiatedTokenAccountBuilder::build()
            .owner(&keypair.pubkey())
            .mint(&mint.pubkey())
            .payer(&keypair.pubkey())
            .instruction();

        // Mint 1 token to the payer's token account.
        let mint_to_payer_ix = TokenExtensionsMintToBuilder::build()
            .mint(&mint.pubkey())
            .account(&payer_token_account)
            .owner(&keypair.pubkey())
            .amount(1)
            .instruction();

        let last_blockhash = client.get_latest_blockhash().await?;

        let tx = TransactionBuilder::build()
            .instruction(create_mint_ix)
            .instruction(init_metadata_pointer_ix)
            .instruction(init_mint_ix)
            .instruction(init_payer_token_account_ix)
            .instruction(mint_to_payer_ix)
            .instruction(create_policy_ix)
            .signer(&keypair)
            .signer(&mint)
            .payer(&keypair.pubkey())
            .recent_blockhash(last_blockhash)
            .transaction();

        client
            .send_and_confirm_transaction_with_spinner(&tx)
            .await?;

        let mut account_data = client.get_account(&address).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy = Policy::deserialize(&mut account_data)?;

        info!("🎉 Policy successfully created! 🎉");
        info!("--------------------------------");
        info!("📜 Policy Details:");
        info!("  📬 Policy Address: {}", address);
        info!("  🪙 Mint Address: {}", mint.pubkey());
        info!("  📈 Strategy: {:?}", policy.strategy);
        info!(
            "  🛡️ Validator Identities: {:?}",
            policy.validator_identities
        );
        info!("--------------------------------");

        Ok(())
    }
}
