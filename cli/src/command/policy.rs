use borsh::BorshDeserialize;
use log::info;
use solana_commitment_config::CommitmentConfig;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions},
    pod::PodMint,
    state::Mint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as MetadataInterfaceBorshDeserialize, state::TokenMetadata,
};
use yellowstone_shield_client::{
    accounts::{Policy, PolicyV2},
    instructions::{ClosePolicyBuilder, CreatePolicyBuilder},
    types::{Kind, PermissionStrategy},
    CreateAccountBuilder, CreateAsscoiatedTokenAccountBuilder, InitializeMetadataBuilder,
    InitializeMint2Builder, MetadataPointerInitializeBuilder, PolicyTrait,
    TokenExtensionsMintToBuilder, TransactionBuilder,
};

use super::{RunCommand, RunResult};
use crate::{command::CommandContext, CommandComplete, LogPolicy, SolanaAccount};

#[derive(Debug)]
pub enum PolicyVersion {
    V1(Policy),
    V2(PolicyV2),
}

impl PolicyVersion {
    pub fn strategy(&self) -> u8 {
        match self {
            PolicyVersion::V1(pv1) => pv1.strategy,
            PolicyVersion::V2(pv2) => pv2.strategy,
        }
    }
}

/// Builder for creating a new policy
pub struct CreateCommandBuilder {
    strategy: Option<PermissionStrategy>,
    name: Option<String>,
    symbol: Option<String>,
    uri: Option<String>,
}

impl Default for CreateCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateCommandBuilder {
    /// Create a new PolicyBuilder
    pub fn new() -> Self {
        Self {
            strategy: None,
            name: None,
            symbol: None,
            uri: None,
        }
    }

    /// Set the strategy for the policy
    pub fn strategy(mut self, strategy: PermissionStrategy) -> Self {
        self.strategy = Some(strategy);
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
impl RunCommand for CreateCommandBuilder {
    /// Execute the creation of the policy
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        // Given a PDA derived from the payer's public key.
        let mint = Keypair::new();
        // Create a token account for the payer.
        let payer_token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            &mint.pubkey(),
            &spl_token_2022::ID,
        );

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
        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(&mint.pubkey());
        let create_policy_ix = CreatePolicyBuilder::new()
            .policy(address)
            .mint(mint.pubkey())
            .payer(keypair.pubkey())
            .owner(keypair.pubkey())
            .token_account(payer_token_account)
            .strategy(self.strategy.expect("strategy must be set"))
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

        let signature = client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .await?;

        info!("Transaction signature: {}", signature);

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(&account_data[..Policy::LEN])?),
            Kind::PolicyV2 => {
                PolicyVersion::V2(PolicyV2::from_bytes(&account_data[..PolicyV2::LEN])?)
            }
        };

        let mint_data = client.get_account(&mint.pubkey()).await?;
        let mint_account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(mint_account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        LogPolicy::new(&mint.pubkey(), &token_metadata, &address, &policy, None).log();

        Ok(CommandComplete(
            SolanaAccount(mint.pubkey(), Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}

/// Builder for deleting a policy
pub struct DeleteCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
}

impl Default for DeleteCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DeleteCommandBuilder<'a> {
    /// Create a new DeleteCommandBuilder
    pub fn new() -> Self {
        Self { mint: None }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for DeleteCommandBuilder<'_> {
    /// Execute the deletion of a policy
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);
        let payer_token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let close_policy = ClosePolicyBuilder::new()
            .policy(address)
            .mint(*mint)
            .payer(keypair.pubkey())
            .owner(keypair.pubkey())
            .token_account(payer_token_account)
            .instruction();

        let last_blockhash = client.get_latest_blockhash().await?;

        let tx = TransactionBuilder::build()
            .instruction(close_policy)
            .signer(&keypair)
            .payer(&keypair.pubkey())
            .recent_blockhash(last_blockhash)
            .transaction();

        let signature = client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .await?;

        info!("Transaction signature: {}", signature);

        Ok(CommandComplete(
            SolanaAccount(*mint, None),
            SolanaAccount(address, None),
        ))
    }
}

pub struct ShowCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
}

impl Default for ShowCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ShowCommandBuilder<'a> {
    pub fn new() -> Self {
        Self { mint: None }
    }

    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for ShowCommandBuilder<'_> {
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair: _, client } = context;

        let mint = self.mint.expect("mint must be set");
        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(account_data)?),
            Kind::PolicyV2 => PolicyVersion::V2(PolicyV2::from_bytes(account_data)?),
        };

        let identities = match policy_version {
            Kind::Policy => Policy::try_deserialize_identities(account_data)?,
            Kind::PolicyV2 => PolicyV2::try_deserialize_identities(account_data)?,
        };

        let mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        LogPolicy::new(&mint, &token_metadata, &address, &policy, Some(&identities)).log();

        Ok(CommandComplete(
            SolanaAccount(*mint, Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}
