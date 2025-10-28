pub mod identity;
pub mod policy;

use {
    crate::policy::PolicyVersion,
    anyhow::Result,
    log::info,
    solana_client::{client_error::ClientError, nonblocking::rpc_client::RpcClient},
    solana_commitment_config::CommitmentConfig,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    spl_token_metadata_interface::state::TokenMetadata,
    yellowstone_shield_client::TransactionBuilder,
};

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
        let instructions: Vec<_> = batch.iter().map(&mut instruction_builder).collect();

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
