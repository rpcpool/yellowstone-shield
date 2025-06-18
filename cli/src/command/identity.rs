use super::{CommandComplete, RunCommand, RunResult, SolanaAccount};
use crate::{command::CommandContext, policy::PolicyVersion};
use borsh::BorshDeserialize;
use log::info;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, PodStateWithExtensions},
    pod::PodMint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as TokenBorshDeserialize, state::TokenMetadata,
};
use yellowstone_shield_client::{
    accounts::{Policy, PolicyV2},
    types::Kind,
};
use yellowstone_shield_client::{
    instructions::{AddIdentityBuilder, RemoveIdentityBuilder},
    TransactionBuilder,
};

const CHUNK_SIZE: usize = 20;

/// Builder for adding a identities to a policy
#[derive(Debug, Clone)]
pub struct AddBatchCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identities: Option<Vec<Pubkey>>,
}

impl Default for AddBatchCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> AddBatchCommandBuilder<'a> {
    /// Create a new AddCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identities: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identities to add
    pub fn identities(mut self, identities: Vec<Pubkey>) -> Self {
        self.identities = Some(identities);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for AddBatchCommandBuilder<'_> {
    /// Execute the addition of a identity to the policy
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");

        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);
        let identities = self.identities.take().expect("identities must be set");
        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let current = match policy_version {
            Kind::Policy => Policy::try_deserialize_identities(&account_data[Policy::LEN..]),
            Kind::PolicyV2 => PolicyV2::try_deserialize_identities(&account_data[PolicyV2::LEN..]),
        }?;

        let add: Vec<Pubkey> = identities
            .into_iter()
            .filter(|identity| !current.contains(identity))
            .collect();

        for batch in add.chunks(CHUNK_SIZE) {
            let mut instructions = Vec::new();
            for identity in batch {
                let add_identity_ix = AddIdentityBuilder::new()
                    .policy(address)
                    .mint(*mint)
                    .token_account(token_account)
                    .payer(keypair.pubkey())
                    .owner(keypair.pubkey())
                    .identity(*identity)
                    .instruction();
                instructions.push(add_identity_ix);
            }

            let last_blockhash = client.get_latest_blockhash().await?;

            let tx = TransactionBuilder::build()
                .instructions(instructions)
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
        }

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(&account_data[..Policy::LEN])?),
            Kind::PolicyV2 => {
                PolicyVersion::V2(PolicyV2::from_bytes(&account_data[..PolicyV2::LEN])?)
            }
        };

        let mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}

/// Builder for removing identities from a policy
pub struct RemoveBatchCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identities: Option<Vec<Pubkey>>,
}

impl Default for RemoveBatchCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RemoveBatchCommandBuilder<'a> {
    /// Create a new RemoveCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identities: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identities to remove
    pub fn identities(mut self, identities: Vec<Pubkey>) -> Self {
        self.identities = Some(identities);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for RemoveBatchCommandBuilder<'_> {
    /// Execute the removal of an identity from the policy
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);
        let identities = self.identities.take().expect("identity must be set");

        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let current = match policy_version {
            Kind::Policy => Policy::try_deserialize_identities(&account_data[Policy::LEN..]),
            Kind::PolicyV2 => PolicyV2::try_deserialize_identities(&account_data[PolicyV2::LEN..]),
        }?;

        let remove: Vec<usize> = identities
            .into_iter()
            .filter_map(|identity| {
                current
                    .iter()
                    .position(|&current_identity| current_identity == identity)
            })
            .collect();

        for batch in remove.chunks(CHUNK_SIZE) {
            let mut instructions = Vec::new();
            for &index in batch {
                let remove_identity_ix = RemoveIdentityBuilder::new()
                    .policy(address)
                    .mint(*mint)
                    .token_account(token_account)
                    .payer(keypair.pubkey())
                    .owner(keypair.pubkey())
                    .index(index as u64)
                    .instruction();
                instructions.push(remove_identity_ix);
            }

            let last_blockhash = client.get_latest_blockhash().await?;

            let tx = TransactionBuilder::build()
                .instructions(instructions)
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
        }

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(&account_data[..Policy::LEN])?),
            Kind::PolicyV2 => {
                PolicyVersion::V2(PolicyV2::from_bytes(&account_data[..PolicyV2::LEN])?)
            }
        };

        let mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}
