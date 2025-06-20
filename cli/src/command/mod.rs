pub mod identity;
pub mod policy;

use anyhow::Result;
use log::info;
use solana_client::client_error::ClientError;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token_metadata_interface::state::TokenMetadata;
use yellowstone_shield_client::TransactionBuilder;

use crate::policy::PolicyVersion;

pub struct CommandContext {
    pub client: RpcClient,
    pub keypair: Keypair,
}

pub struct SolanaAccount<T>(pub Pubkey, pub Option<T>);
pub struct CommandComplete(
    pub SolanaAccount<TokenMetadata>,
    pub SolanaAccount<PolicyVersion>,
);

pub type RunResult = Result<CommandComplete>;

#[async_trait::async_trait]
pub trait RunCommand {
    async fn run(&mut self, context: CommandContext) -> RunResult;
}

async fn send_batched_tx<T, F>(
    client: &RpcClient,
    keypair: &Keypair,
    items: &[T],
    chunk_size: usize,
    mut instruction_builder: F,
) -> Result<(), ClientError>
where
    T: Clone,
    F: FnMut(&T) -> Instruction,
{
    for batch in items.chunks(chunk_size) {
        let instructions: Vec<_> = batch.iter().map(|item| instruction_builder(item)).collect();

        if instructions.is_empty() {
            continue;
        }

        let last_blockhash = client.get_latest_blockhash().await?;

        let tx = TransactionBuilder::build()
            .instructions(instructions)
            .signer(keypair)
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

    Ok(())
}
