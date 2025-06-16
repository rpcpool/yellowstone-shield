pub mod identity;
pub mod policy;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use spl_token_metadata_interface::state::TokenMetadata;
use yellowstone_shield_client::accounts::PolicyV2;

pub struct CommandContext {
    pub client: RpcClient,
    pub keypair: Keypair,
}

pub struct SolanaAccount<T>(pub Pubkey, pub Option<T>);
pub struct CommandComplete(
    pub SolanaAccount<TokenMetadata>,
    pub SolanaAccount<PolicyV2>,
);

pub type RunResult = Result<CommandComplete>;

#[async_trait::async_trait]
pub trait RunCommand {
    async fn run(&mut self, context: CommandContext) -> RunResult;
}
