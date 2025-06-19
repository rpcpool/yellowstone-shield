mod command;

use anyhow::{Context, Result};
use bs58::decode;
use clap_derive::{Parser as DeriveParser, Subcommand};
use serde_json::from_str as parse_json_str;
use solana_cli_config::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::fs::read_to_string as read_path;
use std::path::PathBuf;
use std::sync::Arc;
use std::{str::FromStr, time::Duration};
use yellowstone_shield_client::types::PermissionStrategy;

pub use command::*;

#[derive(Debug, DeriveParser)]
#[command(
    author,
    version,
    name = "Yellowstone Shield CLI",
    about = "CLI for managing Yellowstone shield policies"
)]
pub struct Args {
    /// RPC endpoint url to override using the Solana config
    #[arg(short, long, global = true)]
    pub rpc: Option<String>,

    /// Log level
    #[arg(short, long, global = true, default_value = "off")]
    pub log_level: String,

    /// Path to the owner keypair file
    #[arg(short, long, global = true)]
    pub keypair: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage policies
    Policy {
        #[command(subcommand)]
        action: PolicyAction,
    },
    /// Manage identities
    Identities {
        #[command(subcommand)]
        action: IdentitiesAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PolicyAction {
    /// Create a new policy
    Create {
        /// The strategy to use for the policy
        #[arg(long)]
        strategy: PermissionStrategy,

        /// The name of the policy
        #[arg(long)]
        name: String,

        /// The symbol of the policy
        #[arg(long)]
        symbol: String,

        /// The URI of the policy
        #[arg(long)]
        uri: String,
    },
    /// Delete a policy
    Delete {
        /// The mint address associated with the policy
        #[arg(long)]
        mint: Pubkey,
    },
    /// Show policy details
    Show {
        /// The mint address associated with the policy
        #[arg(long)]
        mint: Pubkey,
    },
}

#[derive(Subcommand, Debug)]
pub enum IdentitiesAction {
    /// Add identities to a policy
    Add {
        /// The mint address associated with the policy
        #[arg(long)]
        mint: Pubkey,
        /// The identities to add to the policy
        #[arg(long)]
        identities_path: PathBuf,
    },
    /// Remove identities from a policy
    Remove {
        /// The mint address associated with the policy
        #[arg(long)]
        mint: Pubkey,
        /// The identities to remove from the policy
        #[arg(long)]
        identities_path: PathBuf,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("unable to get config file path")]
    ConfigFilePathError,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    ParseCommitmentLevelError(#[from] solana_sdk::commitment_config::ParseCommitmentLevelError),
    #[error("unable to parse keypair")]
    Keypair,
}

pub async fn run(config: Arc<Config>, command: Command) -> RunResult {
    let client = RpcClient::new_with_timeout_and_commitment(
        config.json_rpc_url.clone(),
        Duration::from_secs(90),
        CommitmentConfig::from_str(&config.commitment).map_err::<CliError, _>(Into::into)?,
    );
    let keypair = parse_keypair(&config.keypair_path)?;
    let context = command::CommandContext { keypair, client };

    match &command {
        Command::Policy { action } => match action {
            PolicyAction::Create {
                strategy,
                name,
                symbol,
                uri,
            } => {
                policy::CreateCommandBuilder::new()
                    .strategy(*strategy)
                    .name(name.clone())
                    .symbol(symbol.clone())
                    .uri(uri.clone())
                    .run(context)
                    .await
            }
            PolicyAction::Delete { mint } => {
                policy::DeleteCommandBuilder::new()
                    .mint(mint)
                    .run(context)
                    .await
            }
            PolicyAction::Show { mint } => {
                policy::ShowCommandBuilder::new()
                    .mint(mint)
                    .run(context)
                    .await
            }
        },
        Command::Identities { action } => match action {
            IdentitiesAction::Add {
                mint,
                identities_path,
            } => {
                let identities: Vec<Pubkey> = read_path(identities_path)?
                    .lines()
                    .filter_map(|s| Pubkey::from_str(s.trim()).ok())
                    .collect();

                identity::AddBatchCommandBuilder::new()
                    .mint(mint)
                    .identities(identities)
                    .run(context)
                    .await
            }
            IdentitiesAction::Remove {
                mint,
                identities_path,
            } => {
                let identities: Vec<Pubkey> = read_path(identities_path)?
                    .lines()
                    .filter_map(|s| Pubkey::from_str(s.trim()).ok())
                    .collect();

                identity::RemoveBatchCommandBuilder::new()
                    .mint(mint)
                    .identities(identities)
                    .run(context)
                    .await
            }
        },
    }
}

fn parse_keypair(keypair_path: &str) -> Result<Keypair, CliError> {
    let secret_string = read_path(keypair_path).context("Can't find key file")?;
    let secret_bytes = parse_json_str(&secret_string)
        .or_else(|_| decode(&secret_string.trim()).into_vec())
        .map_err(|_| CliError::ConfigFilePathError)?;

    Keypair::from_bytes(&secret_bytes).map_err(|_| CliError::Keypair)
}
