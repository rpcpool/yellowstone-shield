use anyhow::Result;
use log::info;
use solana_sdk::{
    account::WritableAccount,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions},
    pod::PodMint,
    state::Mint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as MetadataInterfaceBorshDeserialize, state::TokenMetadata,
};
use yellowstone_shield_client::{
    accounts::Policy, instructions::CreatePolicyBuilder, types::PermissionStrategy,
    CreateAccountBuilder, CreateAsscoiatedTokenAccountBuilder, InitializeMetadataBuilder,
    InitializeMint2Builder, MetadataPointerInitializeBuilder, TokenExtensionsMintToBuilder,
    TransactionBuilder,
};

use super::RunCommand;
use crate::CommandContext;
use borsh::BorshDeserialize;

/// Builder for creating a new policy
pub struct CreateCommandBuilder<'a> {
    strategy: Option<&'a PermissionStrategy>,
    validator_identities: Option<&'a Vec<Pubkey>>,
    name: Option<String>,
    symbol: Option<String>,
    uri: Option<String>,
}

impl<'a> CreateCommandBuilder<'a> {
    /// Create a new PolicyBuilder
    pub fn new() -> Self {
        Self {
            strategy: None,
            validator_identities: None,
            name: None,
            symbol: None,
            uri: None,
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

    /// Set the name for the token metadata
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the symbol for the token metadata
    pub fn symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// Set the URI for the token metadata
    pub fn uri(mut self, uri: String) -> Self {
        self.uri = Some(uri);
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
        let mint_size =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer])
                .unwrap();

        let token_metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey::try_from(Some(keypair.pubkey())).unwrap(),
            mint: mint.pubkey(),
            name: self.name.clone().expect("name must be set"),
            symbol: self.symbol.clone().expect("symbol must be set"),
            uri: self.uri.clone().expect("uri must be set"),
            additional_metadata: Vec::<(String, String)>::new(),
        };

        let rent = mint_size + token_metadata.tlv_size_of().unwrap();

        let create_mint_ix = CreateAccountBuilder::build()
            .payer(&keypair.pubkey())
            .account(&mint.pubkey())
            .space(mint_size)
            .rent(rent)
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

        let init_metadata_ix = InitializeMetadataBuilder::new()
            .mint(&mint.pubkey())
            .owner(&keypair.pubkey())
            .update_authority(&keypair.pubkey())
            .mint_authority(&keypair.pubkey())
            .name(token_metadata.name)
            .symbol(token_metadata.symbol)
            .uri(token_metadata.uri)
            .instruction();

        // Create the policy account.
        let (address, _) = Policy::find_pda(&mint.pubkey());
        let create_policy_ix = CreatePolicyBuilder::new()
            .policy(address)
            .mint(mint.pubkey())
            .payer(keypair.pubkey())
            .token_account(payer_token_account)
            .validator_identities(validator_identities.to_vec())
            .strategy(yellowstone_shield_client::types::PermissionStrategy::Allow)
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
            .instruction(init_metadata_ix)
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

        let mut mint_data = client.get_account(&mint.pubkey()).await?;
        let account_data: &[u8] = mint_data.data_as_mut_slice();

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(&account_data).unwrap();
        let mut mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(&mut mint_bytes).unwrap();

        info!("🎉 Policy successfully created! 🎉");
        info!("--------------------------------");
        info!("🏠 Addresses:");
        info!("  📜 Policy: {}", address);
        info!("  🔑 Mint: {}", mint.pubkey());
        info!("--------------------------------");
        info!("🔍 Details:");
        match policy.strategy {
            PermissionStrategy::Allow => info!("  ✅ Strategy: Allow"),
            PermissionStrategy::Deny => info!("  ❌ Strategy: Deny"),
        }
        info!(
            "  🛡️ Validator Identities: {:?}",
            policy.validator_identities
        );
        info!("  🏷️ Name: {}", token_metadata.name);
        info!("  🔖 Symbol: {}", token_metadata.symbol);
        info!("  🌐 URI: {}", token_metadata.uri);
        info!("--------------------------------");

        Ok(())
    }
}
