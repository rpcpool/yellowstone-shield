use super::{CommandComplete, RunCommand, RunResult, SolanaAccount};
use crate::command::CommandContext;
use borsh::BorshDeserialize;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signer;
use solana_sdk::{account::WritableAccount, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, PodStateWithExtensions},
    pod::PodMint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as TokenBorshDeserialize, state::TokenMetadata,
};
use yellowstone_shield_client::accounts::Policy;
use yellowstone_shield_client::{
    instructions::{AddIdentityBuilder, RemoveIdentityBuilder},
    TransactionBuilder,
};

/// Builder for adding a identity to a policy
pub struct AddCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identity: Option<&'a Pubkey>,
}

impl Default for AddCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> AddCommandBuilder<'a> {
    /// Create a new AddCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identity: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identity to add
    pub fn identity(mut self, identity: &'a Pubkey) -> Self {
        self.identity = Some(identity);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for AddCommandBuilder<'_> {
    /// Execute the addition of a identity to the policy
    async fn run(&self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        let (address, _) = Policy::find_pda(mint);
        let identity = self.identity.expect("identity must be set");
        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let add_identity_ix = AddIdentityBuilder::new()
            .policy(address)
            .mint(*mint)
            .token_account(token_account)
            .payer(keypair.pubkey())
            .identity(*identity)
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
                CommitmentConfig::finalized(),
            )
            .await?;

        let mut account_data = client.get_account(&address).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy = Policy::deserialize(&mut account_data)?;

        let mut mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = mint_data.data_as_mut_slice();

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, token_metadata),
            SolanaAccount(address, policy),
        ))
    }
}

/// Builder for removing an identity from a policy
pub struct RemoveCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identity: Option<&'a Pubkey>,
}

impl Default for RemoveCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RemoveCommandBuilder<'a> {
    /// Create a new RemoveCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identity: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identity to remove
    pub fn identity(mut self, identity: &'a Pubkey) -> Self {
        self.identity = Some(identity);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for RemoveCommandBuilder<'_> {
    /// Execute the removal of an identity from the policy
    async fn run(&self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        let (address, _) = Policy::find_pda(mint);
        let identity = self.identity.expect("identity must be set");

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
            .identity(*identity)
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
                CommitmentConfig::finalized(),
            )
            .await?;

        let mut account_data = client.get_account(&address).await?;
        let mut account_data: &[u8] = account_data.data_as_mut_slice();

        let policy = Policy::deserialize(&mut account_data)?;

        let mut mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = mint_data.data_as_mut_slice();

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, token_metadata),
            SolanaAccount(address, policy),
        ))
    }
}
